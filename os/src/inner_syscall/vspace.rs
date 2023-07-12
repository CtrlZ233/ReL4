use common::{types::{Pptr, CapRights}, message::InvocationLabel, utils::{convert_to_mut_type_ref, bit, mask, page_bits_for_size, addr_from_pptr}, config::{USER_TOP, PAGE_BITS}};

use crate::{cspace::{CapTableEntry, Cap, CapTag}, mm::{PageTableEntry, find_vspace_for_asid, look_up_pt_slot, look_up_pt_slot2, VmRights, PTEFlags}, scheduler::{ThreadStateEnum, set_thread_state}};
use super::{CUR_EXTRA_CAPS, get_syscall_arg};
use log::{debug, error};
use crate::mm::VMAttributes;

pub fn decode_frame_invocation(label: usize, length: usize, cte: &mut CapTableEntry, cap: Cap,
    _call: bool, buffer: Pptr) {

    match InvocationLabel::from_usize(label) {
        InvocationLabel::PageMap => {
            if length < 3 || unsafe {CUR_EXTRA_CAPS[0] == 0} {
                error!("RISCVPageMap: Truncated message");
                return;
            }

            let vaddr = get_syscall_arg(0, buffer);
            let rights_mask = get_syscall_arg(1, buffer);
            let vm_attributes = VMAttributes::from_word(get_syscall_arg(2, buffer));
            let lvl1pt_cap =unsafe {
                convert_to_mut_type_ref::<CapTableEntry>(CUR_EXTRA_CAPS[0]).cap
            };

            if lvl1pt_cap.get_cap_type() != CapTag::CapPageTableCap || !lvl1pt_cap.get_pt_is_mapped() {
                error!("RISCVPageMap: Bad PageTable cap: {:?}", lvl1pt_cap.get_cap_type());
                return;
            }

            let frame_size = cap.get_frame_size();
            let cap_vm_rights = VmRights::from_usize(cap.get_frame_vm_right());

            let lvl1pt = convert_to_mut_type_ref::<PageTableEntry>(lvl1pt_cap.get_pt_based_ptr());
            let asid = lvl1pt_cap.get_pt_mapped_asid();

            match find_vspace_for_asid(asid) {
                Some(vspace_root) => {
                    if vspace_root as *mut PageTableEntry as usize != lvl1pt as *mut PageTableEntry as usize {
                        error!("RISCVPageMap: ASID lookup failed");
                        return;
                    }

                    let vtop = vaddr + bit(frame_size) - 1;

                    if vtop >= USER_TOP {
                        error!("RISCVPageMap, out of USER TOP");
                        return;
                    }

                    if vaddr & mask(page_bits_for_size(frame_size)) != 0 {
                        error!("RISCVPageMap, AlignmentError");
                        return;
                    }

                    let (bit_left, pte_ptr) = look_up_pt_slot2(lvl1pt, vaddr);
                    let lookup_pte = convert_to_mut_type_ref::<PageTableEntry>(pte_ptr);
                    if bit_left != page_bits_for_size(frame_size) {
                        error!("RISCVPageMap, FailedLookup: {:#x} : {} : {}", vaddr, bit_left, frame_size);
                        return;
                    }

                    let frame_asid = cap.get_frame_mapped_asid();
                    if frame_asid != 0 {
                        if frame_asid != asid {
                            error!("RISCVPageMap: Attempting to remap a frame that does not belong to the passed address space");
                            return;
                        }
                        let map_addr = cap.get_frame_mapped_addr();
                        if map_addr != vaddr {
                            error!("RISCVPageMap: attempting to map frame into multiple addresses");
                            return;
                        }

                        if lookup_pte.is_pte_page_table() {
                            error!("RISCVPageMap: no mapping to remap.");
                            return;
                        }
                    } else {
                        if lookup_pte.is_valid() {
                            error!("RISCVPageMap: Virtual address already mapped");
                            return;
                        }
                    }

                    let vm_rights = cap_vm_rights.mask_vm_rights(CapRights::from_word(rights_mask));
                    let frame_paddr = addr_from_pptr(cap.get_frame_base_ptr());
                    let mut local_cap = cap;
                    local_cap.set_frame_mapped_address(vaddr);
                    local_cap.set_frame_mapped_asid(asid);

                    let executable = !vm_attributes.get_excute_never();
                    let pte = PageTableEntry::make_user_pte(frame_paddr, executable, vm_rights);
                    
                    set_thread_state(ThreadStateEnum::ThreadStateRestart);
                    
                    perform_page_invocation(local_cap, cte, pte, lookup_pte);
                }
                _ => {
                    error!("RISCVPageMap: No PageTable for ASID: {}", asid);
                    return;
                }
            }



        }

        _ => {
            panic!("[decode_frame_invocation] unsupported!");
        }
    }
    
}


pub fn decode_page_table_invocation(label: usize, length: usize, cte: &mut CapTableEntry, cap: Cap, buffer: Pptr) {
    if length < 2 || unsafe { CUR_EXTRA_CAPS[0] == 0 } {
        error!("RISCVPageTable: truncated message");
        return;
    }

    if cap.get_pt_is_mapped() {
        error!("RISCVPageTable: PageTable is already mapped.");
        return;
    }

    let vaddr = get_syscall_arg(0, buffer);
    let lvl1pt_cap =unsafe {
        convert_to_mut_type_ref::<CapTableEntry>(CUR_EXTRA_CAPS[0]).cap
    };

    if lvl1pt_cap.get_cap_type() != CapTag::CapPageTableCap || !lvl1pt_cap.get_pt_is_mapped() {
        error!("RISCVPageTableMap: Invalid top-level PageTable.: {:?}", lvl1pt_cap.get_cap_type());
        return;
    }

    let lvl1pt = convert_to_mut_type_ref::<PageTableEntry>(lvl1pt_cap.get_pt_based_ptr());
    let asid = lvl1pt_cap.get_pt_mapped_asid();

    if vaddr >= USER_TOP {
        error!("RISCVPageTableMap: Virtual address cannot be in kernel window.");
        return;
    }

    match find_vspace_for_asid(asid) {
        Some(vspace_root) => {
            if vspace_root as *mut PageTableEntry as usize != lvl1pt as *mut PageTableEntry as usize {
                error!("RISCVPageTableMap: ASID lookup failed");
                return;
            }

            let (bits_left, pte_ptr) = look_up_pt_slot2(lvl1pt, vaddr);
            let lookup_pte = convert_to_mut_type_ref::<PageTableEntry>(pte_ptr);
            if bits_left == PAGE_BITS || lookup_pte.is_valid() {
                error!("RISCVPageTableMap: All objects mapped at this address");
                return;
            }

            let paddr = addr_from_pptr(cap.get_pt_based_ptr());
            let pte = PageTableEntry::new(paddr >> PAGE_BITS, 0, PTEFlags::V);
            let mut local_cap = cap;
            local_cap.set_pt_is_mapped(1);
            local_cap.set_pt_mapped_asid(asid);
            local_cap.set_pt_mapped_address(vaddr & !mask(bits_left));
            
            set_thread_state(ThreadStateEnum::ThreadStateRestart);
            perform_page_table_invocation(local_cap, cte, pte, lookup_pte);
        }
        _ => {
            error!("RISCVPageTableMap: No PageTable for ASID: {}", asid);
            return;
        }
    }


}

fn perform_page_invocation(cap: Cap, ct_slot: &mut CapTableEntry, pte: PageTableEntry,
    base: &mut PageTableEntry) {

    ct_slot.cap = cap;
    base.update(pte);
}

fn perform_page_table_invocation(cap: Cap, ct_slot: &mut CapTableEntry, pte: PageTableEntry,
    base: &mut PageTableEntry) {

    ct_slot.cap = cap;
    base.update(pte);
}