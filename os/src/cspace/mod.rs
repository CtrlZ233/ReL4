use log::{debug, error};
use crate::config::{CONFIG_ROOT_CNODE_SIZE_BITS, IT_ASID, SEL4_WORD_BITS};
use crate::cspace::cnode::CNodeSlot::{SeL4CapInitThreadCNode, SeL4CapDomain, SeL4CapInitThreadVspace, SeL4CapBootInfoFrame};
use crate::root_server::ROOT_SERVER;
use crate::types::{Pptr, Vptr, VmRights::VMReadWrite};

mod cnode;
mod cap;

pub use cap::{Cap, CapTag, CapTableEntry};
pub use cnode::{CNode, CNodeSlot};
use crate::boot::NDKS_BOOT;
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

pub fn create_bi_frame_cap(cnode_cap: Cap, vptr: Vptr, boot_info_ptr: Pptr) -> Cap {
    create_frame_cap(cnode_cap, vptr, boot_info_ptr, SeL4CapBootInfoFrame as usize, IT_ASID)
}

pub fn create_frame_cap(cnode_cap: Cap, vptr: Vptr, pptr: Pptr, index: usize, asid: usize) -> Cap {
    let cap = Cap::new_frame_cap(asid, pptr,
                                 0, VMReadWrite as usize, false, vptr);
    write_slot(cnode_cap.get_cap_pptr(), index, cap);
    cap
}


pub fn write_slot(cnode_ptr: Pptr, index: usize, cap: Cap) {
    let cnode = unsafe {
        &mut *(cnode_ptr as *mut CNode)
    };
    cnode.write(index, cap);
}
