use crate::config::PPTR_BASE_OFFSET;

pub type Pptr = usize;
pub type Vptr = usize;
pub type Paddr = usize;
pub type Cptr = usize;

pub type SlotPos = usize;

pub type Dom = usize;

pub type NodeId = usize;

pub enum VmRights {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Region {
    pub start: Pptr,
    pub end: Pptr,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct PhyRegion {
    pub start: Paddr,
    pub end: Paddr,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct VirtRegion {
    pub start: Vptr,
    pub end: Vptr,
}
#[derive(Default, Debug, Clone, Copy)]
pub struct SlotRegion {
    pub start: SlotPos,
    pub end: SlotPos,
}

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