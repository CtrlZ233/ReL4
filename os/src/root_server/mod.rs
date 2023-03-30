mod root_server;

use core::cmp::max;
use lazy_static::*;
use log::{debug, error};
use spin::Mutex;
use crate::{config::{CONFIG_ROOT_CNODE_SIZE_BITS, SEL4_SLOT_BITS, SEL4_VSPACE_BITS, SEL4_TCB_BITS, SEL4_PAGE_BITS, BI_FRAME_SIZE_BITS, SEL4_ASID_POOL_BITS}, mm::Region, utils::bit, vspace::get_n_paging};
use crate::boot::NDKS_BOOT;
use crate::config::MAX_NUM_FREEMEM_REG;
use crate::cspace::{CapTag, create_root_cnode};
use crate::mm::VirtRegion;
use crate::types::Pptr;
use crate::utils::round_down;

use self::root_server::RootServer;

lazy_static! {
    static ref ROOT_SERVER_MEM: Mutex<Region> = Mutex::new(Region::default());
    pub static ref ROOT_SERVER: Mutex<RootServer> = Mutex::new(RootServer::default());
}

pub fn root_server_init(it_v_reg: VirtRegion, extra_bi_size_bits: usize) {
    let mut i = (MAX_NUM_FREEMEM_REG - 1) as isize;
    let mut ndks_boot = NDKS_BOOT.lock();
    if ndks_boot.freemem[i as usize].start != ndks_boot.freemem[i as usize].end {
        error!("insufficient MAX_NUM_FREEMEM_REG: {}", MAX_NUM_FREEMEM_REG);
        assert_eq!(1, 0);
    }

    while i >= 0 && ndks_boot.freemem[i as usize].start == ndks_boot.freemem[i as usize].end {
        i -= 1;
    }
    let size = calculate_root_server_size(it_v_reg, extra_bi_size_bits);
    let max_bits = root_server_max_size_bits(extra_bi_size_bits);
    while i >= 0 {
        let index = i  as usize;
        assert_eq!(ndks_boot.freemem[index + 1].start, ndks_boot.freemem[index + 1].end);

        let empty_index = index + 1;
        let unaligned_start = ndks_boot.freemem[index].end - size;
        let start = round_down(unaligned_start, max_bits);
        if unaligned_start <= ndks_boot.freemem[index].end && start >= ndks_boot.freemem[index].start {
            create_root_server_objects(start, it_v_reg, extra_bi_size_bits);
            ndks_boot.freemem[empty_index] = Region {
                start: start + size,
                end: ndks_boot.freemem[index].end,
            };
            ndks_boot.freemem[index].end = start;
            return;
        }
        ndks_boot.freemem[empty_index] = ndks_boot.freemem[index];
        ndks_boot.freemem[index] = Region::default();
        i -= 1;
    }

    let root_cnode_cap = create_root_cnode();
    if root_cnode_cap.get_cap_type() == CapTag::CapNullCap {
        error!("root c-node creation failed");
        assert_eq!(1, 0);
    }

    debug!("no free memory region is big enough for root server");
    assert_eq!(1, 0);
}

fn root_server_max_size_bits(extra_bi_size_bits: usize) -> usize {
    let cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS + SEL4_SLOT_BITS;
    let max_bits = max(cnode_size_bits, SEL4_VSPACE_BITS);
    max(max_bits, extra_bi_size_bits)
}

fn calculate_root_server_size(it_v_reg: VirtRegion, extra_bi_size_bits: usize) -> usize {
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
    size += bit(SEL4_VSPACE_BITS);

    size + get_n_paging(it_v_reg) * bit(SEL4_PAGE_BITS)
}

fn create_root_server_objects(start: usize, it_v_reg: VirtRegion, extra_bi_size_bits: usize) {
    let cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS + SEL4_SLOT_BITS;
    let max_bits = root_server_max_size_bits(extra_bi_size_bits);

    let size = calculate_root_server_size(it_v_reg, extra_bi_size_bits);
    let mut root_server_mm = ROOT_SERVER_MEM.lock();

    root_server_mm.start = start;
    root_server_mm.end = start + size;
    drop(root_server_mm);

    maybe_alloc_extra_bi(max_bits, extra_bi_size_bits);
    ROOT_SERVER.lock().cnode = alloc_root_server_obj(cnode_size_bits, 1);
    maybe_alloc_extra_bi(SEL4_VSPACE_BITS, extra_bi_size_bits);
    ROOT_SERVER.lock().vspace = alloc_root_server_obj(SEL4_VSPACE_BITS, 1);
    maybe_alloc_extra_bi(SEL4_VSPACE_BITS, extra_bi_size_bits);
    ROOT_SERVER.lock().asid_pool = alloc_root_server_obj(SEL4_ASID_POOL_BITS, 1);
    ROOT_SERVER.lock().ipc_buf = alloc_root_server_obj(SEL4_PAGE_BITS, 1);
    ROOT_SERVER.lock().boot_info = alloc_root_server_obj(BI_FRAME_SIZE_BITS, 1);

    let n = get_n_paging(it_v_reg);
    let start = alloc_root_server_obj(SEL4_PAGE_BITS, n);
    let end = start + n * bit(SEL4_PAGE_BITS);
    ROOT_SERVER.lock().paging = Region {start, end};

    ROOT_SERVER.lock().tcb = alloc_root_server_obj(SEL4_TCB_BITS, 1);

    {
        let root_server_mm = ROOT_SERVER_MEM.lock();
        assert_eq!(root_server_mm.start, root_server_mm.end);
    }
}

fn alloc_root_server_obj(size_bits: usize, n: usize) -> Pptr {
    let mut root_server_mem = ROOT_SERVER_MEM.lock();
    let allocated = root_server_mem.start;
    let len = n * bit(size_bits);
    assert_eq!(allocated % bit(size_bits), 0);
    root_server_mem.start += len;
    assert!(root_server_mem.start <= root_server_mem.end);
    (allocated as usize..(allocated + len) as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    allocated
}

fn maybe_alloc_extra_bi(cmp_size_bits: usize, extra_bi_size_bits: usize) {
    if extra_bi_size_bits >= cmp_size_bits && ROOT_SERVER.lock().extra_bi == 0 {
        ROOT_SERVER.lock().extra_bi = alloc_root_server_obj(extra_bi_size_bits, 1);
    }
}