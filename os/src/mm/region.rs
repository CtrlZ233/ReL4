use crate::config::PPTR_BASE_OFFSET;

#[derive(Default, Debug, Clone, Copy)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}


pub type PhyRegion = Region;

pub type VirtRegion = Region;

impl Region {
    pub fn paddr_to_pptr_reg(p_reg: PhyRegion) -> Self {
        Region { start: p_reg.start + PPTR_BASE_OFFSET, end: p_reg.end + PPTR_BASE_OFFSET }
    }
}

impl PhyRegion {
    pub fn pptr_to_paddr_reg(reg: Region) -> Self {
        PhyRegion {
            start: reg.start - PPTR_BASE_OFFSET,
            end: reg.start - PPTR_BASE_OFFSET,
        }
    }
}

