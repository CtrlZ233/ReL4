mod init_freemem;
mod ndks_boot;
mod boot_info;

use crate::config::{NUM_RESERVED_REGIONS, MAX_NUM_FREEMEM_REG, AVAIL_PHY_MEM_START, AVAIL_PHY_MEM_END, AVAIL_MEM_DEVICE};
use crate::types::{Region, PhyRegion, VirtRegion, APPtr, ASIDSizeConstants};
use lazy_static::*;
use spin::Mutex;
use ndks_boot::NdksBoot;
use crate::config::{BI_FRAME_SIZE_BITS, PAGE_BITS, UI_P_REG_END, UI_P_REG_START, UI_PV_OFFSET, USER_TOP};
use crate::utils::bit;

pub use boot_info::{calculate_extra_bi_size_bits, BootInfo, BootInfoID, BootInfoHeader};
use crate::types::Vptr;

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
                    ui_vp_offset: isize) {
    crate::root_server::init(it_v_reg, extra_bi_size_bits, ipc_buf_vptr, extra_bi_size, extra_bi_offset,
                             bi_frame_vptr, extra_bi_frame_vptr, ui_reg, ui_vp_offset);
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
    root_server_init(it_v_reg, extra_bi_size_bits, ipc_buf_vptr, extra_bi_size, extra_bi_offset,
                     bi_frame_vptr, extra_bi_frame_vptr, ui_reg, UI_PV_OFFSET as isize);
}

