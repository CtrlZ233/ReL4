mod init_freemem;
mod ndks_boot;

use crate::{mm::Region, config::{NUM_RESERVED_REGIONS, MAX_NUM_FREEMEM_REG, AVAIL_PHY_MEM_START, AVAIL_PHY_MEM_END, AVAIL_MEM_DEVICE}};
use lazy_static::*;
use spin::Mutex;
use ndks_boot::NdksBoot;

lazy_static!{
    static ref RES_REG: Mutex<[Region; NUM_RESERVED_REGIONS]> = Mutex::new([Region::default(); NUM_RESERVED_REGIONS]);

    static ref NDKS_BOOT: Mutex<NdksBoot> = Mutex::new(NdksBoot::default());

    static ref AVAIL_P_REGS: Mutex<[Region; AVAIL_MEM_DEVICE]> = Mutex::new([Region{start: AVAIL_PHY_MEM_START, end: AVAIL_PHY_MEM_END}; AVAIL_MEM_DEVICE]);

    static ref AVAIL_REG: Mutex<[Region; MAX_NUM_FREEMEM_REG]> = Mutex::new([Region::default(); MAX_NUM_FREEMEM_REG]);
}


pub fn boot_mem_init() {
    let ui_reg = Region { start: 0xffffffc084020000, end: 0xffffffc0840203a1 };
    init_freemem::init(ui_reg, Region::default());
}