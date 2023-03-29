use crate::{types::Pptr, mm::Region};

#[derive(Default, Debug)]
pub struct RootServer {
    cnode: Pptr,
    vspace: Pptr,
    asid_pool: Pptr,
    ipc_buf: Pptr,
    boot_info: Pptr,
    extra_bi: Pptr,
    tcb: Pptr,
    paging: Region,
}