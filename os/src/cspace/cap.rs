use common::config::{CONFIG_RESET_CHUNK_BITS, MIN_UNTYPED_BITS, SEL4_ASID_POOL_BITS, SEL4_ENDPOINT_BITS,
    SEL4_NOTIFICATION_BITS, SEL4_PAGE_BITS, SEL4_SLOT_BITS, SEL4_TCB_BITS, WORD_BITS};
use common::types::Pptr;
use common::utils::{bit, mask, page_bits_for_size, round_down, convert_to_mut_type_ref};
use log::debug;

use super::cap_data::CapData;
use super::mdb::MDBNode;

#[derive(Clone, Copy)]
pub struct Cap {
    pub words: [usize; 2],
}

#[derive(Copy, Clone)]
pub struct CapTableEntry {
    pub(crate) cap: Cap,
    pub(crate) mdb_node: MDBNode,
}

impl CapTableEntry {
    pub fn ensure_empty_slot(&self) -> bool {
        self.cap.get_cap_type() == CapTag::CapNullCap
    }

    pub fn ensure_no_child(&self) -> bool {
        if self.mdb_node.get_mdb_next() != 0 {
            let next = unsafe {
                &mut *(self.mdb_node.get_mdb_next() as *mut CapTableEntry)
            };
            if self.is_mdb_parent_of(next) {
                return false;
            }
        }
        true
    }

    pub fn reset_untyped_cap(&mut self) -> bool {
        assert_eq!(self.cap.get_cap_type(), CapTag::CapUntypedCap);
        let prev_cap = self.cap;
        let block_size = prev_cap.get_untyped_cap_block_size();
        let region_base = prev_cap.get_untyped_ptr();
        let chunk = CONFIG_RESET_CHUNK_BITS;
        let offset = prev_cap.get_untyped_free_index() << MIN_UNTYPED_BITS;
        let device_mem = prev_cap.get_untyped_is_device();
        if offset == 0 {
            return true;
        }
        if device_mem || block_size < chunk {
            if !device_mem {
                let end = region_base + bit(block_size);
                (region_base as usize..end as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
            }
            self.cap.set_untyped_cap_free_index(0);
        } else {
            let mut local_offset = round_down(offset - 1, chunk);
            debug!("local_offset: {}, region_base: {:#x}", local_offset, region_base);
            let stride = bit(chunk);
            loop {
                let start = region_base + local_offset;
                let end = region_base + bit(chunk);
                (start as usize..end as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
                self.cap.set_untyped_cap_free_index((local_offset as usize) >> MIN_UNTYPED_BITS);
                // TODO: preemption point
                local_offset -= stride;
                if local_offset == 0 {
                    break;
                }
            }
        }
        true
    }

    pub fn is_mdb_parent_of(&self, other: &CapTableEntry) -> bool {
        if !self.mdb_node.get_mdb_revocable() {
            return false;
        }

        if !self.cap.same_region_as(&other.cap) {
            return false;
        }

        match self.cap.get_cap_type() {
            CapTag::CapEndpointCap => {
                let badge = self.cap.get_ep_badge();
                if badge == 0 {
                    return true;
                }
                return badge == other.cap.get_ep_badge() && !other.mdb_node.get_mdb_first_badged();
            }

            CapTag::CapNotificationCap => {
                let badge = self.cap.get_nt_fn_badge();
                if badge == 0 {
                    return true;
                }
                return badge == other.cap.get_nt_fn_badge() && !other.mdb_node.get_mdb_first_badged();
            }

            _ => {

            }
        }

        true
    }

    pub fn is_final_cap(&self) -> bool {
        let mdb = self.mdb_node;
        let prev_is_same_obj;
        if mdb.get_mdb_prev() == 0 {
            prev_is_same_obj = false;
        } else {
            let prev = convert_to_mut_type_ref::<CapTableEntry>(mdb.get_mdb_prev());
            prev_is_same_obj = prev.cap.same_obj_as(&self.cap);
        }

        if prev_is_same_obj {
            return false;
        } else {
            if mdb.get_mdb_next() == 0 {
                return true;
            } else {
                let next = convert_to_mut_type_ref::<CapTableEntry>(mdb.get_mdb_next());
                return !self.cap.same_obj_as(&next.cap);
            }
        }
    }

    pub fn is_long_running_delete(&self) -> bool {
        if self.cap.get_cap_type() == CapTag::CapNullCap || !self.is_final_cap() {
            return false;
        }
        match self.cap.get_cap_type() {
            CapTag::CapThreadCap | CapTag::CapZombieCap | CapTag::CapCNodeCap => true,
            _ => false
        }
    }

    pub fn delete(&mut self, exposed: bool) -> bool {
        if let Some(cleanup_info) = self.finalise_slot(exposed) {
            self.emplty_slot(cleanup_info);
            return true;
        }
        if exposed {
            return true;
        }
        return false;
    }

    pub fn emplty_slot(&mut self, cleanup_info: Cap) {
        if self.cap.get_cap_type() != CapTag::CapNullCap {
            let mdb_node = self.mdb_node;
            if mdb_node.get_mdb_prev() != 0 {
                let prev = convert_to_mut_type_ref::<CapTableEntry>(mdb_node.get_mdb_prev());
                prev.mdb_node.set_mdb_next(mdb_node.get_mdb_next());
            }

            if mdb_node.get_mdb_next() != 0 {
                let next = convert_to_mut_type_ref::<CapTableEntry>(mdb_node.get_mdb_next());
                next.mdb_node.set_mdb_prev(mdb_node.get_mdb_prev());
                next.mdb_node.set_mdb_first_badged(next.mdb_node.get_mdb_first_badged() || mdb_node.get_mdb_first_badged());
            }
            
            self.cap = Cap::new_null_cap();
            self.mdb_node = MDBNode::null_mdbnode();
            // postCapDeletion need to do;
        }
    }

    pub fn finalise_slot(&mut self, immediate: bool) -> Option<Cap>{

        while self.cap.get_cap_type() != CapTag::CapNullCap {
            let is_final = self.is_final_cap();
            let fc_ret = finalise_cap(self.cap, is_final, false);
            if is_cap_removable(fc_ret.remainder, self) {
                return Some(fc_ret.cleanup_info);
            }
            
            panic!("failed to finalise slot");
        }
        Some(Cap::new_null_cap())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum CapTag {
    CapNullCap = 0,
    CapUntypedCap = 2,
    CapEndpointCap = 4,
    CapNotificationCap = 6,
    CapReplyCap = 8,
    CapCNodeCap = 10,
    CapThreadCap = 12,
    CapIrqControlCap = 14,
    CapIrqHandlerCap = 16,
    CapZombieCap = 18,
    CapDomainCap = 20,
    CapFrameCap = 1,
    CapPageTableCap = 3,
    CapASIDControlCap = 11,
    CapASIDPoolCap = 13
}

impl Cap {
    pub fn get_cap_type(&self) -> CapTag {
        unsafe {
            core::mem::transmute::<u8, CapTag>(((self.words[0] >> 59) & 0x1f) as u8)
        }
    }

    pub fn update_cap_data(&mut self, preserve: bool, new_data: usize) {
        match self.get_cap_type() {
            CapTag::CapEndpointCap => {
                if !preserve && self.get_ep_badge() == 0 {
                    self.set_ep_badge(new_data);
                }
            }

            CapTag::CapNotificationCap => {
                if !preserve && self.get_nt_fn_badge() == 0 {
                    self.set_nt_fn_badge(new_data);
                }
            }

            CapTag::CapCNodeCap => {
                let w = CapData::new(new_data);
                let guard_size = w.get_guard_size();
                if guard_size +  self.get_cnode_radix() <= WORD_BITS {
                    let guard = w.get_guard() & mask(guard_size);
                    self.set_cnode_guard(guard);
                    self.set_cnode_guard_size(guard_size);
                }
            }
            _ => {}
        }
    }

    pub fn same_obj_as(&self, other: &Self) -> bool {
        if self.get_cap_type() == CapTag::CapUntypedCap {
            return false;
        }

        if self.get_cap_type() == CapTag::CapIrqControlCap && other.get_cap_type() == CapTag::CapIrqHandlerCap {
            return false;
        }

        if self.get_cap_type() == CapTag::CapFrameCap && other.get_cap_type() == CapTag::CapFrameCap {
            return self.get_frame_base_ptr() == other.get_frame_base_ptr() &&
                    self.get_frame_size() == other.get_frame_size() &&
                    self.get_frame_is_device() == other.get_frame_is_device();
        }
        return self.same_region_as(other);
    }

    pub fn same_region_as(&self, other: &Self) -> bool {
        match self.get_cap_type() {
            CapTag::CapUntypedCap => {
                if other.is_physical() {
                    let self_base = self.get_cap_pptr();
                    let other_base = other.get_cap_pptr();

                    let self_top = self_base + mask(self.get_untyped_cap_block_size());
                    let other_top = other_base + mask(other.get_cap_size_bits());
                    return self_base <= other_base && other_top <= self_top && other_base <= other_top;
                }
            }

            CapTag::CapEndpointCap | CapTag::CapNotificationCap | CapTag::CapThreadCap |
            CapTag::CapPageTableCap | CapTag::CapASIDPoolCap => {
                if other.get_cap_type() == self.get_cap_type() {
                    return self.get_cap_pptr() == other.get_cap_pptr();
                }
            }

            CapTag::CapCNodeCap => {
                if other.get_cap_type() == CapTag::CapCNodeCap {
                    return self.get_cnode_ptr() == other.get_cnode_ptr() && self.get_cnode_radix() == other.get_cnode_radix();
                }
            }

            CapTag::CapReplyCap => {
                if other.get_cap_type() == CapTag::CapReplyCap {
                    return self.get_reply_tcb_ptr() == other.get_reply_tcb_ptr();
                }
            }

            CapTag::CapDomainCap | CapTag::CapASIDControlCap => {
                return other.get_cap_type() == self.get_cap_type();
            }

            CapTag::CapIrqControlCap => {
                return other.get_cap_type() == CapTag::CapIrqControlCap || other.get_cap_type() == CapTag::CapIrqHandlerCap;
            }

            CapTag::CapIrqHandlerCap => {
                if other.get_cap_type() == CapTag::CapIrqHandlerCap {
                    return self.get_irq_handler() == other.get_irq_handler();
                }
            }

            CapTag::CapFrameCap => {
                if other.get_cap_type() == CapTag::CapFrameCap {
                    let bot_a = self.get_frame_base_ptr();
                    let bot_b = other.get_frame_base_ptr();
                    let top_a = bot_a + mask(page_bits_for_size(self.get_frame_size()));
                    let top_b = bot_b + mask(page_bits_for_size(other.get_frame_size()));
                    return bot_a <= bot_b  && top_a >= top_b && bot_b <= top_b;
                }
            }

            _ => {
                return false;
            }
        }
        false
    }

    pub fn is_physical(&self) -> bool {
        match self.get_cap_type() {
            CapTag::CapUntypedCap | CapTag::CapEndpointCap | CapTag::CapNotificationCap |
            CapTag::CapCNodeCap | CapTag::CapThreadCap | CapTag::CapZombieCap | CapTag::CapFrameCap |
            CapTag::CapPageTableCap | CapTag::CapASIDPoolCap => {
                true
            }
            _ => false
        }
    }

    pub fn get_cap_pptr(&self) -> Pptr {
        match self.get_cap_type() {
            CapTag::CapUntypedCap => self.get_untyped_ptr(),
            CapTag::CapCNodeCap => self.get_cnode_ptr(),
            CapTag::CapPageTableCap => self.get_pt_based_ptr(),
            CapTag::CapASIDPoolCap => self.get_asid_pool(),
            CapTag::CapFrameCap => self.get_frame_base_ptr(),
            CapTag::CapNotificationCap => self.get_nt_fn_ptr(),
            CapTag::CapEndpointCap => self.get_ep_ptr(),
            CapTag::CapThreadCap => self.get_tcb_ptr(),
            _ => { panic!("invalid type") }
        }
    }

    pub fn get_cap_size_bits(&self) -> usize {
        match self.get_cap_type() {
            CapTag::CapUntypedCap => self.get_untyped_cap_block_size(),
            CapTag::CapEndpointCap => SEL4_ENDPOINT_BITS,
            CapTag::CapNotificationCap => SEL4_NOTIFICATION_BITS,
            CapTag::CapCNodeCap => self.get_cnode_radix() + SEL4_SLOT_BITS,
            CapTag::CapThreadCap => SEL4_TCB_BITS,
            CapTag::CapZombieCap => {
                panic!("invalid type")
            }
            CapTag::CapFrameCap => page_bits_for_size(self.get_frame_size()),
            CapTag::CapPageTableCap => SEL4_PAGE_BITS,
            CapTag::CapASIDPoolCap => SEL4_ASID_POOL_BITS,
            _ => 0,
        }
    }
}


pub fn is_cap_revocable(derived_cap: Cap, src_cap: Cap) -> bool {
    return match derived_cap.get_cap_type() {
        CapTag::CapUntypedCap => {
            true
        }
        CapTag::CapEndpointCap => {
            derived_cap.get_ep_badge() != src_cap.get_ep_badge()
        }
        CapTag::CapNotificationCap => {
            derived_cap.get_nt_fn_badge() != src_cap.get_nt_fn_badge()
        }
        CapTag::CapIrqHandlerCap => {
            src_cap.get_cap_type() == CapTag::CapIrqControlCap
        }
        _ => {
            false
        }
    }
}

pub fn is_cap_removable(cap: Cap, slot: &CapTableEntry) -> bool {
    match cap.get_cap_type() {
        CapTag::CapNullCap => {
            return true;
        }

        _ => {
            panic!()
        }
    }
}

#[derive(Clone, Copy)]
struct FinaliseCapRet {
    pub remainder: Cap,
    pub cleanup_info: Cap,
}

fn finalise_cap(cap: Cap, is_final: bool, exposed: bool) -> FinaliseCapRet {
    match cap.get_cap_type() {
        _ => {
            FinaliseCapRet {
                remainder: Cap::new_null_cap(),
                cleanup_info: Cap::new_null_cap(),
            }
        }
    }
}