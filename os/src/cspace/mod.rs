use crate::config::{CONFIG_ROOT_CNODE_SIZE_BITS, SEL4_WORD_BITS};
use crate::cspace::cnode::CNodeSlot::SeL4CapInitThreadCNode;
use crate::root_server::ROOT_SERVER;
use crate::types::Pptr;

mod cnode;
mod cap;

pub use cap::{Cap, CapTag, CapTableEntry};
pub use cnode::{CNode, CNodeSlot};

pub fn create_root_cnode() -> Cap {
    let cap = Cap::new_cnode_cap(CONFIG_ROOT_CNODE_SIZE_BITS,
                                 SEL4_WORD_BITS - CONFIG_ROOT_CNODE_SIZE_BITS,
                                 0,
                                 ROOT_SERVER.lock().cnode as usize);
    write_slot(ROOT_SERVER.lock().cnode, SeL4CapInitThreadCNode as usize, cap);
    cap
}

pub fn write_slot(cnode_ptr: Pptr, index: usize, cap: Cap) {
    let cnode = unsafe {
        &mut *(cnode_ptr as *mut CNode)
    };
    cnode.write(index, cap);
}