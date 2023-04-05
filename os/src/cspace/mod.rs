use log::{debug, error};
use crate::config::{CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS, CONFIG_ROOT_CNODE_SIZE_BITS, IT_ASID, MAX_UNTYPED_BITS, MIN_UNTYPED_BITS, SEL4_WORD_BITS, WORD_BITS};
use crate::cspace::cnode::CNodeSlot::{SeL4CapInitThreadCNode, SeL4CapDomain, SeL4CapInitThreadVspace, SeL4CapBootInfoFrame, SeL4CapInitThreadASIDPool, SeL4CapASIDControl};
use crate::root_server::ROOT_SERVER;
use crate::types::{ASIDSizeConstants, Region, SlotPos};
use crate::types::{Pptr, Vptr, VmRights::VMReadWrite};

mod cnode;
mod cap;

pub use cap::{Cap, CapTag, CapTableEntry, MDBNode};
pub use cnode::{CNode, CNodeSlot, TCBCNodeIndex};
use crate::boot::NDKS_BOOT;
use crate::cspace::cap::{is_cap_revocable};
use crate::cspace::CapTag::CapPageTableCap;
use crate::cspace::CNodeSlot::SeL4CapInitThreadTcb;
use crate::untyped::set_untyped_cap_as_full;
use crate::utils::bit;

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
    write_slot(cnode_cap.get_cap_pptr(), SeL4CapInitThreadTcb as usize, cap);
    cap
}

pub fn create_bi_frame_cap(cnode_cap: Cap, vptr: Vptr, boot_info_ptr: Pptr) -> Cap {
    create_frame_cap(cnode_cap, vptr, boot_info_ptr, SeL4CapBootInfoFrame as usize, IT_ASID)
}

pub fn create_frame_cap(cnode_cap: Cap, vptr: Vptr, pptr: Pptr, index: usize, asid: usize) -> Cap {
    let cap = Cap::new_frame_cap(asid, pptr,
                                 0, VMReadWrite as usize, false, vptr);
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

pub fn derive_cap(slot: &mut CapTableEntry, cap: Cap) -> (bool, Cap) {
    return match cap.get_cap_type() {
        CapTag::CapFrameCap => {
            let mut new_cap = cap;
            new_cap.frame_cap_set_frame_mapped_address(0);
            (true, new_cap)
        }
        _ => {
            (false, Cap::new_null_cap())
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

