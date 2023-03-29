mod root_server;

use core::cmp::max;
use lazy_static::*;
use spin::Mutex;
use crate::{config::{CONFIG_ROOT_CNODE_SIZE_BITS, SEL4_SLOT_BITS, SEL4_VSPACE_BITS, SEL4_TCB_BITS, SEL4_PAGE_BITS, BI_FRAME_SIZE_BITS, SEL4_ASID_POOL_BITS}, mm::Region, utils::bit, vspace::get_n_paging};

use self::root_server::RootServer;

lazy_static! {
    static ref ROOT_SERVER_MEM: Mutex<Region> = Mutex::new(Region::default());
    static ref ROOT_SERVER: Mutex<RootServer> = Mutex::new(RootServer::default());
}

pub fn rootserver_max_size_bits(extra_bi_size_bits: usize) -> usize {
    let cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS + SEL4_SLOT_BITS;
    let max_bits = max(cnode_size_bits, SEL4_VSPACE_BITS);
    max(max_bits, extra_bi_size_bits)
}

pub fn calculate_rootserver_size(it_v_reg: Region, extra_bi_size_bits: usize) -> usize {
    let mut size = bit(CONFIG_ROOT_CNODE_SIZE_BITS + SEL4_SLOT_BITS);
    size += bit(SEL4_TCB_BITS); // root thread tcb
    size += bit(SEL4_PAGE_BITS); // ipc buf
    size += bit(BI_FRAME_SIZE_BITS); // boot info
    size += bit(SEL4_ASID_POOL_BITS);
    size += if extra_bi_size_bits > 0 {
        bit(extra_bi_size_bits)
    } else {
        0
    };

    size + get_n_paging(it_v_reg) * bit(SEL4_PAGE_BITS)
}

pub fn create_rootserver_objects(start: usize, it_v_reg: Region, extra_bi_size_bits: usize) {
    let cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS + SEL4_SLOT_BITS;
    let max_bits = rootserver_max_size_bits(extra_bi_size_bits);

    let size = calculate_rootserver_size(it_v_reg, extra_bi_size_bits);

    let mut root_server_mm = ROOT_SERVER_MEM.lock();
    let mut root_server = ROOT_SERVER.lock();

    root_server_mm.start = start;
    root_server_mm.end = start + size;
    


}