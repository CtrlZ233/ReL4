
use crate::types::{Region, PhyRegion};
use crate::config::{MAX_NUM_RESV_REG, MAX_NUM_FREEMEM_REG};
use crate::types::{Pptr, SlotPos};

#[derive(Default)]
pub struct NdksBoot {
    pub reserved: [PhyRegion; MAX_NUM_RESV_REG],
    pub resv_count: usize,
    pub freemem: [Region; MAX_NUM_FREEMEM_REG],
    pub boot_info_ptr: Pptr,
    pub slot_pos_cur: SlotPos,
}

