mod init_freemem;
mod ndks_boot;
mod boot_info;

use crate::config::{NUM_RESERVED_REGIONS, MAX_NUM_FREEMEM_REG, AVAIL_PHY_MEM_START, AVAIL_PHY_MEM_END, AVAIL_MEM_DEVICE, CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS, WORD_BITS, MAX_UNTYPED_BITS, MIN_UNTYPED_BITS, CONFIG_PADDR_USER_DEVICE_TOP, CONFIG_ROOT_CNODE_SIZE_BITS};
use crate::types::{Region, PhyRegion, VirtRegion, APPtr, ASIDSizeConstants, SlotPos, Pptr, SlotRegion};
use lazy_static::*;
use log::debug;
use spin::Mutex;
use ndks_boot::NdksBoot;
use crate::config::{BI_FRAME_SIZE_BITS, PAGE_BITS, UI_P_REG_END, UI_P_REG_START, UI_PV_OFFSET, USER_TOP};
use crate::utils::bit;

pub use boot_info::{calculate_extra_bi_size_bits, BootInfo, BootInfoID, BootInfoHeader};
use crate::cspace::Cap;
use crate::types::Vptr;
use crate::untyped::create_untyped_for_region;

lazy_static!{
    static ref RES_REG: Mutex<[Region; NUM_RESERVED_REGIONS]> = Mutex::new([Region::default(); NUM_RESERVED_REGIONS]);

    pub static ref NDKS_BOOT: Mutex<NdksBoot> = Mutex::new(NdksBoot::default());

    static ref AVAIL_P_REGS: Mutex<[PhyRegion; AVAIL_MEM_DEVICE]> = Mutex::new([PhyRegion{start: AVAIL_PHY_MEM_START, end: AVAIL_PHY_MEM_END}; AVAIL_MEM_DEVICE]);

    static ref AVAIL_REG: Mutex<[Region; MAX_NUM_FREEMEM_REG]> = Mutex::new([Region::default(); MAX_NUM_FREEMEM_REG]);

    static ref KS_ASID_TABLE: Mutex<[APPtr; 1 << (ASIDSizeConstants::ASIDHighBits as usize)]> = Mutex::new([0; 1 << (ASIDSizeConstants::ASIDHighBits as usize)]);
}


fn boot_mem_init(ui_reg: Region) {
    init_freemem::init(ui_reg);
}

fn root_server_init(it_v_reg: VirtRegion, extra_bi_size_bits: usize, ipc_buf_vptr: Vptr, extra_bi_size: usize,
                    extra_bi_offset: usize, bi_frame_vptr: Vptr, extra_bi_frame_vptr: Vptr, ui_reg: Region,
                    ui_vp_offset: isize) -> Cap {
    crate::root_server::init(it_v_reg, extra_bi_size_bits, ipc_buf_vptr, extra_bi_size, extra_bi_offset,
                             bi_frame_vptr, extra_bi_frame_vptr, ui_reg, ui_vp_offset)
}

fn create_untypeds(root_cnode_cap: Cap) {
    let first_untyped_slot = NDKS_BOOT.lock().slot_pos_cur;

    let mut start = 0;
    let ndks_boot_resv_count = NDKS_BOOT.lock().resv_count;
    for i in 0..ndks_boot_resv_count {
        let resv_start = NDKS_BOOT.lock().reserved[i].start;
        if start < resv_start {
            let reg = Region::paddr_to_pptr_reg(
                PhyRegion {
                    start,
                    end: resv_start,
                }
            );
            create_untyped_for_region(root_cnode_cap, true, reg, first_untyped_slot);
        }
        start = NDKS_BOOT.lock().reserved[i].end;
    }

    for i in 0..MAX_NUM_FREEMEM_REG {
        let reg = NDKS_BOOT.lock().freemem[i];
        NDKS_BOOT.lock().freemem[i] = Region {start: 0, end:0};
        create_untyped_for_region(root_cnode_cap, false, reg, first_untyped_slot);
    }
    let boot_info = unsafe {
        &mut *(NDKS_BOOT.lock().boot_info_ptr as *mut BootInfo)
    };
    boot_info.untyped = SlotRegion {
        start: first_untyped_slot,
        end: NDKS_BOOT.lock().slot_pos_cur,
    };

    debug!("boot_info_untyped: {:?}", boot_info.untyped);
}

fn boot_info_finalise() {
    let boot_info = unsafe {
        &mut *(NDKS_BOOT.lock().boot_info_ptr as *mut BootInfo)
    };
    boot_info.empty = SlotRegion {
        start: NDKS_BOOT.lock().slot_pos_cur,
        end: bit(CONFIG_ROOT_CNODE_SIZE_BITS),
    }
}

pub fn init() {
    let ui_reg = Region::paddr_to_pptr_reg(PhyRegion {
        start: UI_P_REG_START,
        end: UI_P_REG_END,
    });

    let ui_v_reg = VirtRegion {
        start: UI_P_REG_START - UI_PV_OFFSET,
        end: UI_P_REG_END - UI_PV_OFFSET,
    };

    let ipc_buf_vptr = ui_v_reg.end;
    let bi_frame_vptr = ipc_buf_vptr + bit(PAGE_BITS);
    let extra_bi_frame_vptr = bi_frame_vptr + bit(BI_FRAME_SIZE_BITS);
    let extra_bi_size = core::mem::size_of::<BootInfoHeader>();
    let extra_bi_offset: usize = 0;

    let extra_bi_size_bits = calculate_extra_bi_size_bits(extra_bi_size);

    let it_v_reg = VirtRegion {
        start: ui_v_reg.start,
        end: extra_bi_frame_vptr + bit(extra_bi_size_bits),
    };
    assert!(it_v_reg.end < USER_TOP);

    boot_mem_init(ui_reg);
    let root_cnode_cap = root_server_init(it_v_reg, extra_bi_size_bits, ipc_buf_vptr, extra_bi_size, extra_bi_offset,
                     bi_frame_vptr, extra_bi_frame_vptr, ui_reg, UI_PV_OFFSET as isize);
    create_untypeds(root_cnode_cap);
    
    boot_info_finalise();
}

