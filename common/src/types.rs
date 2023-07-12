use super::config::{PPTR_BASE_OFFSET, NUM_ASID_POOL_BITS, ASID_POOL_INDEX_BITS, SEL4_MSG_MAX_LEN, SEL4_MSG_MAX_EXTRA_CAPS};
use super::utils::bool2usize;
use super::message::MessageInfo;

pub type Pptr = usize;
pub type Vptr = usize;
pub type Paddr = usize;
pub type Cptr = usize;
pub type Prio = usize;

pub type SlotPos = usize;

pub type Dom = usize;

pub type NodeId = usize;

pub type PTEPtr = Pptr;
pub type APPtr = Pptr;

pub enum VMAttributes {
    ExecuteNever = 0x1,
    DefaultVMAttributes = 0x0,
}

pub enum ASIDSizeConstants {
    ASIDHighBits = NUM_ASID_POOL_BITS as isize,
    ASIDLowBits = ASID_POOL_INDEX_BITS as isize,
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
            end: reg.end - PPTR_BASE_OFFSET,
        }
    }
}


const PADDING_LEN: usize = 8 - 2 * 1;
#[derive(Debug, Clone, Copy)]
pub struct UntypedDesc {
    pub paddr: Paddr,
    pub size_bits: u8,
    pub is_device: u8,
    pub padding: [u8; PADDING_LEN],
}

impl UntypedDesc {
    pub fn new(paddr: Paddr, size_bits: usize, is_device: bool) -> Self {
        UntypedDesc {
            paddr,
            size_bits: size_bits as u8,
            is_device: bool2usize(is_device) as u8,
            padding: [0; PADDING_LEN]
        }
    }
}


pub enum CNodeSlot {
    SeL4CapNull =  0,                   /* null cap */
    SeL4CapInitThreadTcb =  1,          /* initial thread's TCB cap */
    SeL4CapInitThreadCNode =  2,        /* initial thread's root CNode cap */
    SeL4CapInitThreadVspace =  3,       /* initial thread's VSpace cap */
    SeL4CapIrqControl =  4,             /* global IRQ controller cap */
    SeL4CapASIDControl =  5,            /* global ASID controller cap */
    SeL4CapInitThreadASIDPool =  6,     /* initial thread's ASID pool cap */
    SeL4CapIOPortControl =  7,          /* global IO port control cap (null cap if not supported) */
    SeL4CapIOSpace =  8,                /* global IO space cap (null cap if no IOMMU support) */
    SeL4CapBootInfoFrame =  9,          /* bootinfo frame cap */
    SeL4CapInitThreadIpcBuffer = 10,    /* initial thread's IPC buffer frame cap */
    SeL4CapDomain = 11,                 /* global domain controller cap */
    SeL4CapSMMUSIDControl = 12,         /*global SMMU SID controller cap, null cap if not supported*/
    SeL4CapSMMUCBControl = 13,          /*global SMMU CB controller cap, null cap if not supported*/
    SeL4NumInitialCaps = 14
}


pub struct IpcBuffer {
    pub tag: MessageInfo,
    pub msg: [usize; SEL4_MSG_MAX_LEN],
    pub user_data: usize,
    pub caps_or_badges: [usize; SEL4_MSG_MAX_EXTRA_CAPS],
    pub receive_cnode: Cptr,
    pub receive_index: Cptr,
    pub receive_depth: usize,
}

pub struct CapRights {
    pub word: [usize; 1],
}

impl CapRights {
    pub fn new(allow_grant_reply: usize, allow_grant: usize, allow_read: usize, allow_write: usize) -> Self {
        let mut rights: CapRights = CapRights { word: [0] };
        rights.word[0] = 0 | (allow_grant_reply & 0x1) << 3
                        | (allow_grant & 0x1) << 2
                        | (allow_read & 0x1) << 1
                        | (allow_write & 0x1) << 0;
        return rights;
    }

    pub fn from_word(w: usize) -> Self {
        Self {
            word: [w]
        }
    }

    pub fn re_stucture(&mut self, allow_grant_reply: usize, allow_grant: usize, allow_read: usize, allow_write: usize) {
        self.word[0] = 0 | (allow_grant_reply & 0x1) << 3
                        | (allow_grant & 0x1) << 2
                        | (allow_read & 0x1) << 1
                        | (allow_write & 0x1) << 0;
    }

    pub fn get_allow_grant_reply(&self) -> bool {
        Self::sign_extend((self.word[0] & 0x8) >> 3, 0x0) != 0
    }

    pub fn set_allow_grant_reply(&mut self, v64: usize) {
        self.word[0] &= !0x8;
        self.word[0] |= (v64 << 3) & 0x8;
    }

    pub fn get_allow_grant(&self) -> bool {
        Self::sign_extend((self.word[0] & 0x4) >> 2, 0x0) != 0
    }

    pub fn set_allow_grant(&mut self, v64: usize) {
        self.word[0] &= !0x4;
        self.word[0] |= (v64 << 2) & 0x4;
    }

    pub fn get_allow_read(&self) -> bool {
        Self::sign_extend((self.word[0] & 0x2) >> 1, 0x0) != 0
    }

    pub fn set_allow_read(&mut self, v64: usize) {
        self.word[0] &= !0x2;
        self.word[0] |= (v64 << 1) & 0x2;
    }

    pub fn get_allow_write(&self) -> bool {
        Self::sign_extend((self.word[0] & 0x1) >> 0, 0x0) != 0
    }

    pub fn set_allow_write(&mut self, v64: usize) {
        self.word[0] &= !0x1;
        self.word[0] |= (v64 << 0) & 0x1;
    }

    #[inline]
    pub fn sign_extend(ret: usize, sign: usize) -> usize {
        if ret & (1 << 63) != 0 {
            return ret | sign;
        }
        ret
    }
}
