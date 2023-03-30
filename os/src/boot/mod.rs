mod init_freemem;
mod ndks_boot;
mod boot_info;

use crate::{mm::{Region, PhyRegion, VirtRegion}, config::{NUM_RESERVED_REGIONS, MAX_NUM_FREEMEM_REG, AVAIL_PHY_MEM_START, AVAIL_PHY_MEM_END, AVAIL_MEM_DEVICE}};
use lazy_static::*;
use spin::Mutex;
use ndks_boot::NdksBoot;
use crate::boot::boot_info::{BootInfoHeader, calculate_extra_bi_size_bits};
use crate::config::{BI_FRAME_SIZE_BITS, PAGE_BITS, UI_P_REG_END, UI_P_REG_START, UI_PV_OFFSET, USER_TOP};
use crate::utils::bit;

lazy_static!{
    static ref RES_REG: Mutex<[Region; NUM_RESERVED_REGIONS]> = Mutex::new([Region::default(); NUM_RESERVED_REGIONS]);

    pub static ref NDKS_BOOT: Mutex<NdksBoot> = Mutex::new(NdksBoot::default());

    static ref AVAIL_P_REGS: Mutex<[PhyRegion; AVAIL_MEM_DEVICE]> = Mutex::new([PhyRegion{start: AVAIL_PHY_MEM_START, end: AVAIL_PHY_MEM_END}; AVAIL_MEM_DEVICE]);

    static ref AVAIL_REG: Mutex<[Region; MAX_NUM_FREEMEM_REG]> = Mutex::new([Region::default(); MAX_NUM_FREEMEM_REG]);
}


fn boot_mem_init(ui_reg: Region) {
    init_freemem::init(ui_reg);
}

fn root_server_init(it_v_reg: VirtRegion, extra_bi_size_bits: usize) {
    crate::root_server::root_server_init(it_v_reg, extra_bi_size_bits);
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

    let ipcbuf_vptr = ui_v_reg.end;
    let bi_frame_vptr = ipcbuf_vptr + bit(PAGE_BITS);
    let extra_bi_frame_vptr = bi_frame_vptr + bit(BI_FRAME_SIZE_BITS);
    let extra_bi_size = core::mem::size_of::<BootInfoHeader>();

    let extra_bi_size_bits = calculate_extra_bi_size_bits(extra_bi_size);

    let it_v_reg = VirtRegion {
        start: ui_v_reg.start,
        end: extra_bi_frame_vptr + bit(extra_bi_size_bits),
    };
    assert!(it_v_reg.end < USER_TOP);

    boot_mem_init(ui_reg);
    root_server_init(it_v_reg, extra_bi_size_bits);

}