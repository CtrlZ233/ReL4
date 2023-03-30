
pub const PHYS_BASE_RAW: usize = 0x80000000;

pub const PADDR_BASE: usize = 0;

pub const PPTR_BASE: usize = 0xFFFFFFC000000000;
pub const PPTR_TOP: usize =  0xFFFFFFFF80000000;

pub const KERNEL_ELF_PADDR_BASE: usize = PHYS_BASE_RAW + 0x200000;
pub const KERNEL_ELF_BASE: usize = PPTR_TOP + (KERNEL_ELF_PADDR_BASE & ((1 << 30) - 1));

pub const PPTR_BASE_OFFSET: usize = PPTR_BASE - PADDR_BASE;
pub const PV_BASE_OFFSET: usize = PPTR_TOP - PHYS_BASE_RAW;

pub const PAGE_SIZE: usize = 0x1000;
pub const CONFIG_PT_LEVELS: usize = 3;
pub const PAGE_TABLE_INDEX_BITS: usize = 9;
pub const PAGE_BITS: usize = 12;
pub const ROOT_PAGE_TABLE_SIZE: usize = 1 << PAGE_TABLE_INDEX_BITS;
pub const SATP_MODE_SV39: usize = 8;

pub const AVAIL_MEM_DEVICE: usize = 1;
pub const AVAIL_PHY_MEM_START: usize = 0x8000_0000;
pub const AVAIL_PHY_MEM_END: usize = 0x8800_0000;

pub const NUM_RESERVED_REGIONS: usize = 3;
pub const MAX_NUM_FREEMEM_REG: usize = 16;
pub const MAX_NUM_RESV_REG: usize = NUM_RESERVED_REGIONS + MAX_NUM_FREEMEM_REG;

pub const SEL4_SLOT_BITS: usize = 5;
pub const SEL4_VSPACE_BITS: usize = PAGE_BITS;
pub const SEL4_TCB_BITS: usize = 10;
pub const SEL4_PAGE_BITS: usize = 12;
pub const BI_FRAME_SIZE_BITS: usize = PAGE_BITS;
pub const SEL4_ASID_POOL_BITS: usize = 12;
pub const SEL4_WORD_BITS: usize = 64;


pub const CONFIG_ROOT_CNODE_SIZE_BITS: usize = 13;

// root server image
pub const UI_P_REG_START: usize = 0x84020000;
pub const UI_P_REG_END: usize = 0x84427000;
pub const UI_PV_OFFSET: usize = 0x84010000;
pub const UI_V_ENTRY: usize = 0x1b932;
pub const USER_TOP: usize = 0x0000003FFFFFF000;


