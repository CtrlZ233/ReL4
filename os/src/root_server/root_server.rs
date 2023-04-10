use common::types::{Pptr, Region};

#[derive(Default, Debug)]
pub struct RootServer {
    pub cnode: Pptr,
    pub vspace: Pptr,
    pub asid_pool: Pptr,
    pub ipc_buf: Pptr,
    pub boot_info: Pptr,
    pub extra_bi: Pptr,
    pub tcb: Pptr,
    pub paging: Region,
}