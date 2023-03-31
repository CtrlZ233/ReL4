mod root_server;

use core::cmp::max;
use core::sync::atomic::Ordering::SeqCst;
use lazy_static::*;
use log::{debug, error};
use spin::Mutex;
use crate::{config::{CONFIG_ROOT_CNODE_SIZE_BITS, SEL4_SLOT_BITS, SEL4_VSPACE_BITS, SEL4_TCB_BITS, SEL4_PAGE_BITS, BI_FRAME_SIZE_BITS, SEL4_ASID_POOL_BITS}, utils::bit};
use crate::boot::{BootInfo, BootInfoHeader, BootInfoID, NDKS_BOOT};
use crate::config::{CONFIG_MAX_NUM_NODES, CONFIG_PT_LEVELS, IT_ASID, MAX_NUM_FREEMEM_REG, PAGE_BITS, PPTR_BASE, ROOT_PAGE_TABLE_SIZE};
use crate::cspace::{Cap, CapTag, CNodeSlot, create_bi_frame_cap, create_domain_cap, create_frame_cap, create_it_pt_cap, create_page_table_cap, create_root_cnode};
use crate::cspace::CNodeSlot::SeL4CapInitThreadIpcBuffer;
use crate::mm::{copy_global_mappings, get_n_paging, map_frame_cap, map_it_pt_cap, PageTableEntry};
use crate::scheduler::{KS_DOM_SCHEDULE, KS_DOM_SCHEDULE_IDX};
use crate::types::{NodeId, Pptr, Vptr, SlotRegion, VirtRegion, Region};
use crate::utils::{get_lvl_page_size, get_lvl_page_size_bits, round_down};

use self::root_server::RootServer;

lazy_static! {
    static ref ROOT_SERVER_MEM: Mutex<Region> = Mutex::new(Region::default());
    pub static ref ROOT_SERVER: Mutex<RootServer> = Mutex::new(RootServer::default());
}

pub fn init(it_v_reg: VirtRegion, extra_bi_size_bits: usize, ipc_buf_vptr: Vptr, extra_bi_size: usize,
            extra_bi_offset: usize, bi_frame_vptr: Vptr, extra_bi_frame_vptr: Vptr) {
    root_server_init(it_v_reg, extra_bi_size_bits);
    populate_bi_frame(0, CONFIG_MAX_NUM_NODES, ipc_buf_vptr, extra_bi_size_bits, extra_bi_size);

    init_boot_info_header(extra_bi_size, extra_bi_offset);

    create_all_caps(it_v_reg, bi_frame_vptr, extra_bi_size, extra_bi_frame_vptr, ipc_buf_vptr);
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

    debug!("no free memory region is big enough for root server");
    assert_eq!(1, 0);
}

fn create_all_caps(it_v_reg: VirtRegion, bi_frame_vptr: Vptr, extra_bi_size: usize,
                   extra_bi_frame_vptr: Vptr,ipc_buf_vptr: Vptr) {
    let root_cnode_cap = create_root_cnode();
    if root_cnode_cap.get_cap_type() == CapTag::CapNullCap {
        error!("root c-node creation failed");
        assert_eq!(1, 0);
    }
    create_domain_cap(root_cnode_cap);
    let it_vspace_cap = create_it_address_space(root_cnode_cap, it_v_reg).unwrap();

    let bi_frame_cap = create_bi_frame_cap(root_cnode_cap, bi_frame_vptr, ROOT_SERVER.lock().boot_info);
    map_frame_cap(it_vspace_cap, bi_frame_cap);

    match maybe_create_extra_bi_frame_cap(root_cnode_cap, it_vspace_cap, extra_bi_size, extra_bi_frame_vptr) {
        None => {}
        Some(region) => {
            let boot_info = unsafe {
                &mut *(NDKS_BOOT.lock().boot_info_ptr as *mut BootInfo)
            };
            boot_info.extra_bi_pages = region;
        }
    }

    let ipc_buf_ptr = ROOT_SERVER.lock().ipc_buf;
    (ipc_buf_ptr as usize..(ipc_buf_ptr + bit(PAGE_BITS)) as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    let ipc_buf_cap = create_frame_cap(root_cnode_cap, ipc_buf_vptr, ROOT_SERVER.lock().ipc_buf,
                                      SeL4CapInitThreadIpcBuffer as usize, IT_ASID);
    map_frame_cap(it_vspace_cap, ipc_buf_cap);

}

fn maybe_create_extra_bi_frame_cap(root_cnode_cap: Cap, vspace_cap: Cap, extra_bi_size: usize,
                                   extra_bi_frame_vptr: Vptr) -> Option<SlotRegion> {
    if extra_bi_size > 0 {
        let mut start = ROOT_SERVER.lock().extra_bi;
        let end = start + extra_bi_size;
        let pv_offset = (extra_bi_frame_vptr as isize) - ((start - PPTR_BASE) as isize);
        let slot_before = NDKS_BOOT.lock().slot_pos_cur;
        while start < end {
            let frame_cap = create_frame_cap(root_cnode_cap, ((start - PPTR_BASE) as isize + pv_offset) as usize,
                                             start,NDKS_BOOT.lock().slot_pos_cur, IT_ASID);
            NDKS_BOOT.lock().slot_pos_cur += 1;
            map_frame_cap(vspace_cap, frame_cap);
            start += bit(PAGE_BITS);
        }
        let slot_after = NDKS_BOOT.lock().slot_pos_cur;
        return Some(SlotRegion {
            start: slot_before,
            end: slot_after,
        });
    }
    None
}




fn create_it_address_space(root_cnode_cap: Cap, it_v_reg: VirtRegion) -> Option<Cap> {
    let vspace = unsafe {
        &mut *(ROOT_SERVER.lock().vspace as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };
    copy_global_mappings(vspace);
    let vspace_ptr = ROOT_SERVER.lock().vspace;
    let lvl1pt_cap = create_page_table_cap(root_cnode_cap,
                                           IT_ASID,
                                           vspace_ptr,
                                           true,
                                           vspace_ptr);
    let slot_before = NDKS_BOOT.lock().slot_pos_cur;

    for i in 0..(CONFIG_PT_LEVELS - 1) {
        let mut pt_vptr = round_down(it_v_reg.start, get_lvl_page_size_bits(i)) as Vptr;
        while pt_vptr < it_v_reg.end {
            let cap = create_it_pt_cap(root_cnode_cap, it_alloc_paging(), pt_vptr, IT_ASID);
            map_it_pt_cap(lvl1pt_cap, cap);
            pt_vptr += get_lvl_page_size(i);
        }
    }

    let slot_after = NDKS_BOOT.lock().slot_pos_cur;
    let bi_frame = unsafe {
        &mut *(NDKS_BOOT.lock().boot_info_ptr as *mut BootInfo)
    };
    bi_frame.user_image_paging = SlotRegion {
        start: slot_before,
        end: slot_after,
    };
    Some(lvl1pt_cap)
}

fn populate_bi_frame(node_id: NodeId, num_nodes: usize, ipc_buf_vptr: Vptr, extra_bi_size_bits: usize, extra_bi_size: usize) {
    // clear boot info mem
    let mut start = ROOT_SERVER.lock().boot_info;
    let mut end = start + bit(BI_FRAME_SIZE_BITS);
    (start as usize..end as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });

    if extra_bi_size_bits != 0 {
        start = ROOT_SERVER.lock().extra_bi;
        end = start + bit(extra_bi_size_bits);
        (start as usize..end as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
    }

    let boot_info = unsafe {
        &mut *(ROOT_SERVER.lock().boot_info as *mut BootInfo)
    };
    boot_info.node_id = node_id;
    boot_info.num_nodes = num_nodes;
    boot_info.num_io_pt_levels = 0;
    boot_info.ipc_buf_ptr = ipc_buf_vptr;
    boot_info.init_thread_cnode_size_bits = CONFIG_ROOT_CNODE_SIZE_BITS;
    boot_info.init_thread_domain = KS_DOM_SCHEDULE.lock()[KS_DOM_SCHEDULE_IDX.load(SeqCst)].domain;
    boot_info.extra_len = extra_bi_size;

    NDKS_BOOT.lock().boot_info_ptr = boot_info as *const BootInfo as Pptr;
    NDKS_BOOT.lock().slot_pos_cur = CNodeSlot::SeL4NumInitialCaps as usize;
    unsafe {
        let ndks_boot = NDKS_BOOT.lock();
        let boot_info = &*(ndks_boot.boot_info_ptr as *const BootInfo);
        debug!("node_id: {}, num_nodes: {}, num_io_pt_levels: {}, ipc_buf_ptr: {:#x},\
            init_thread_cnode_size_bits: {}, init_thread_domain: {}, extra_len: {}", boot_info.node_id,
            boot_info.num_nodes, boot_info.num_io_pt_levels, boot_info.ipc_buf_ptr, boot_info.init_thread_cnode_size_bits,
            boot_info.init_thread_domain, boot_info.extra_len);
    }
}

fn init_boot_info_header(extra_bi_size: usize, extra_bi_offset: usize) {
    let mut header: BootInfoHeader = BootInfoHeader { id: 0, len: 0 };
    if extra_bi_size > extra_bi_offset {
        header.id = BootInfoID::Sel4BootInfoHeaderPadding as usize;
        header.len = extra_bi_size - extra_bi_offset;
        let bih = unsafe {
            &mut *((ROOT_SERVER.lock().extra_bi + extra_bi_offset) as *mut BootInfoHeader)
        };
        bih.id = header.id;
        bih.len = header.len;
    }
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
        let root_server = ROOT_SERVER.lock();
        debug!("root_server.cnode: {:#x}", root_server.cnode);
        debug!("root_server.vspace: {:#x}", root_server.vspace);
        debug!("root_server.asid_pool: {:#x}", root_server.asid_pool);
        debug!("root_server.ipc_buf: {:#x}", root_server.ipc_buf);
        debug!("root_server.boot_info: {:#x}", root_server.boot_info);
        debug!("root_server.extra_bi: {:#x}", root_server.extra_bi);
        debug!("root_server.paging: {:#x} ... {:#x}", root_server.paging.start, root_server.paging.end);
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

fn it_alloc_paging() -> Pptr {
    // debug!("test8");
    let allocated = ROOT_SERVER.lock().paging.start;
    ROOT_SERVER.lock().paging.start += bit(PAGE_BITS);
    // assert!(ROOT_SERVER.lock().paging.start <=  ROOT_SERVER.lock().paging.end);

    allocated
}