use log::{debug, error};
use common::config::{CONFIG_ROOT_CNODE_SIZE_BITS, IT_ASID, SEL4_WORD_BITS, WORD_BITS, WORD_RADIX};
use common::types::CNodeSlot::{SeL4CapInitThreadCNode, SeL4CapDomain, SeL4CapInitThreadVspace, SeL4CapBootInfoFrame, SeL4CapInitThreadASIDPool, SeL4CapASIDControl};
use crate::root_server::ROOT_SERVER;
use common::types::{ASIDSizeConstants, SlotPos, Pptr, Vptr, CNodeSlot, Cptr};

mod cnode;
mod cap;
mod cap_data;
mod cap_fn;
mod mdb;
pub use cap::{Cap, CapTag, CapTableEntry};
pub use cnode::{CNode, TCBCNodeIndex};
use crate::boot::NDKS_BOOT;
use crate::cspace::cap::is_cap_revocable;
use crate::cspace::CapTag::CapCNodeCap;
use crate::untyped::set_untyped_cap_as_full;
use crate::mm::VmRights;
use common::utils::{bit, mask};
pub use mdb::MDBNode;

pub fn create_root_cnode() -> Cap {
    let cap = Cap::new_cnode_cap(CONFIG_ROOT_CNODE_SIZE_BITS,
                                 SEL4_WORD_BITS - CONFIG_ROOT_CNODE_SIZE_BITS,
                                 0,
                                 ROOT_SERVER.lock().cnode as usize);
    write_slot(ROOT_SERVER.lock().cnode, SeL4CapInitThreadCNode as usize, cap);
    cap
}

pub fn create_domain_cap(cnode_cap: Cap) -> Cap {
    let cap = Cap::new_domain_cap();
    debug!("root_cnode_cap.get_cap_pptr: {:#x}", cnode_cap.get_cap_pptr());
    write_slot(cnode_cap.get_cap_pptr(), SeL4CapDomain as usize, cap);
    cap
}

pub fn create_page_table_cap(cnode_cap: Cap, asid: usize, pt_base_ptr: Pptr, is_mapped: bool, pt_mapped_addr: Vptr) -> Cap {
    let cap = Cap::new_page_table_cap(asid, pt_base_ptr, is_mapped, pt_mapped_addr);
    write_slot(cnode_cap.get_cap_pptr(), SeL4CapInitThreadVspace as usize, cap);
    cap
}

pub fn create_it_pt_cap(cnode_cap: Cap, pptr: Pptr, vptr: Vptr, asid: usize) -> Cap {
    let cap = Cap::new_page_table_cap(asid, pptr, true, vptr);
    if NDKS_BOOT.lock().slot_pos_cur >= bit(CONFIG_ROOT_CNODE_SIZE_BITS) {
        error!("can't add another cap, all slot used!");
        assert_eq!(1, 0);
    }
    write_slot(cnode_cap.get_cap_pptr(), NDKS_BOOT.lock().slot_pos_cur, cap);
    NDKS_BOOT.lock().slot_pos_cur += 1;
    cap
}

pub fn create_init_thread_cap(cnode_cap: Cap, tcb_ptr: Pptr) -> Cap {
    let cap = Cap::new_thread_cap(tcb_ptr);
    write_slot(cnode_cap.get_cap_pptr(), CNodeSlot::SeL4CapInitThreadTcb as usize, cap);
    cap
}

pub fn create_bi_frame_cap(cnode_cap: Cap, vptr: Vptr, boot_info_ptr: Pptr) -> Cap {
    create_frame_cap(cnode_cap, vptr, boot_info_ptr, SeL4CapBootInfoFrame as usize, IT_ASID)
}

pub fn create_frame_cap(cnode_cap: Cap, vptr: Vptr, pptr: Pptr, index: usize, asid: usize) -> Cap {
    let cap = Cap::new_frame_cap(asid, pptr,
                                 0, VmRights::VMReadWrite as usize, false, vptr);
    write_slot(cnode_cap.get_cap_pptr(), index, cap);
    cap
}

pub fn create_asid_pool_cap(cnode_cap: Cap, asid: usize, asid_pool_ptr: Pptr) -> Cap {
    let cap = Cap::new_asid_pool_cap(asid >> (ASIDSizeConstants::ASIDLowBits as usize), asid_pool_ptr);
    write_slot(cnode_cap.get_cap_pptr(), SeL4CapInitThreadASIDPool as usize, cap);
    cap
}

pub fn create_asid_control_cap(cnode_cap: Cap) -> Cap {
    let cap = Cap::new_asid_control_cap();
    write_slot(cnode_cap.get_cap_pptr(), SeL4CapASIDControl as usize, cap);
    cap
}

pub fn create_untyped_cap(cnode_cap: Cap, slot: SlotPos, free_index: usize, is_device: bool, size_bits: usize, pptr: Pptr) -> Cap {
    let cap = Cap::new_untyped_cap(free_index, is_device, size_bits, pptr);
    write_slot(cnode_cap.get_cap_pptr(), slot, cap);
    cap
}

pub fn write_slot(cnode_ptr: Pptr, index: usize, cap: Cap) {
    let cnode = unsafe {
        &mut *(cnode_ptr as *mut CNode)
    };
    cnode.write(index, cap);
}

pub fn derive_cap(_slot: &mut CapTableEntry, cap: Cap) -> (bool, Cap) {
    return match cap.get_cap_type() {
        CapTag::CapFrameCap => {
            let mut new_cap = cap;
            new_cap.set_frame_mapped_address(0);
            (true, new_cap)
        }

        CapTag::CapUntypedCap | CapTag::CapZombieCap | CapTag::CapIrqControlCap | CapTag::CapReplyCap => {
            error!("[derive_cap] unsupported: {:?}", cap.get_cap_type());
            (false, Cap::new_null_cap())
        }

        _ => {
            // error!("[derive_cap] unsupported: {:?}", cap.get_cap_type());
            (true, cap)
        }
    }
}

pub fn cte_insert(new_cap: Cap, src_slot: &mut CapTableEntry, dest_slot: &mut CapTableEntry) {
    let mut new_mdb = src_slot.mdb_node;
    let src_cap = src_slot.cap;

    let new_cap_is_revocable = is_cap_revocable(new_cap, src_cap);
    new_mdb.set_mdb_prev(src_cap.get_cap_pptr() as *const CapTableEntry as Pptr);
    new_mdb.set_mdb_revocable(new_cap_is_revocable);
    new_mdb.set_mdb_first_badged(new_cap_is_revocable);

    set_untyped_cap_as_full(src_cap, new_cap, src_slot);

    dest_slot.cap = new_cap;
    dest_slot.mdb_node = new_mdb;
    let ref_src_mdb = &mut src_slot.mdb_node;
    ref_src_mdb.set_mdb_next(dest_slot as *const CapTableEntry as Pptr);

    if new_mdb.get_mdb_next() != 0 {
        let prev_of_new_mdb = unsafe {
            &mut (&mut *(new_mdb.get_mdb_next() as *mut CapTableEntry)).mdb_node
        };
        prev_of_new_mdb.set_mdb_prev(dest_slot as *const CapTableEntry as Pptr);
    }
}

pub fn resolve_address_bits(node_cap: Cap, cap_ptr: usize, n_bits: usize) -> Option<*mut CapTableEntry> {
    if node_cap.get_cap_type() != CapCNodeCap {
        error!("cptr: {}, type: {:?}", cap_ptr, node_cap.get_cap_type());
        return None;
    }
    let mut local_n_bits = n_bits;
    let mut local_node_cap = node_cap;
    loop {
        let radix_bits = local_node_cap.get_cnode_radix();
        let guard_bits = local_node_cap.get_cnode_guard_size();
        let level_bits = radix_bits + guard_bits;
        assert_ne!(level_bits, 0);

        let cap_guard = local_node_cap.get_cnode_guard();
        let guard = (cap_ptr >> ((local_n_bits - guard_bits) & mask(WORD_RADIX))) & mask(guard_bits);
        if guard_bits > local_n_bits || guard != cap_guard {
            return None;
        }

        if level_bits > local_n_bits {
            return None;
        }
        let offset = (cap_ptr >> (local_n_bits - level_bits)) & mask(radix_bits);
        let slot = unsafe {
            &mut (&mut *(local_node_cap.get_cap_pptr() as *mut CNode))[offset]
        };
        if local_n_bits == level_bits {
            return Some(slot as *mut CapTableEntry);
        }
        local_n_bits -= level_bits;
        local_node_cap =  slot.cap;

        if local_node_cap.get_cap_type() != CapCNodeCap {
            return Some(slot as *mut CapTableEntry);
        }
    }
}

pub fn lookup_slot_for_cnode_op(_is_source: bool, root: Cap, cap_ptr: Cptr, depth: usize) -> Option<*mut CapTableEntry> {
    if root.get_cap_type() != CapCNodeCap || depth < 1 || depth > WORD_BITS {
        return None;
    }

    resolve_address_bits(root, cap_ptr, depth)
}

pub fn lookup_target_slot(root: Cap, cap_ptr: Cptr, depth: usize) -> Option<*mut CapTableEntry> {
    lookup_slot_for_cnode_op(false, root, cap_ptr, depth)
}

pub fn insert_new_cap(parent: &mut CapTableEntry, slot: &mut CapTableEntry, cap: Cap) {
    let next = parent.mdb_node.get_mdb_next();
    slot.cap = cap;
    slot.mdb_node = MDBNode::new(next, true, true, parent as *mut CapTableEntry as usize);
    if next != 0 {
        unsafe {
            (&mut *(next as *mut CapTableEntry)).mdb_node.set_mdb_prev(slot as *mut CapTableEntry as usize);
        }
    }
    parent.mdb_node.set_mdb_next(slot as *mut CapTableEntry as usize);
}
