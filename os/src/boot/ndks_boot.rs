
use crate::mm::{Region, PhyRegion};
use crate::config::{MAX_NUM_RESV_REG, MAX_NUM_FREEMEM_REG};

#[derive(Default)]
pub struct NdksBoot {
    pub reserved: [PhyRegion; MAX_NUM_RESV_REG],
    pub resv_count: usize,
    pub freemem: [Region; MAX_NUM_FREEMEM_REG],
    pub sel4_boot_info_vaddr: usize, 
    pub sel4_slot_pos: usize,
}