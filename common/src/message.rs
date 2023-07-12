use crate::register::Register;

pub const NUM_MSG_REGISTRES: usize = 4;
pub const NUM_FRAME_REGISTERS: usize = 16;
pub const NUM_GP_REGISTERS: usize = 16;
pub const NUM_EXCEPTION_MSG: usize = 2;
pub const NUM_SYSCALL_MSG: usize = 10;

pub const MESSAGE_REGISTERS: [usize; NUM_MSG_REGISTRES] = [
    Register::a2 as usize,
    Register::a3 as usize,
    Register::a4 as usize,
    Register::a5 as usize,
];

pub const FRAME_REGISTERS: [usize; NUM_FRAME_REGISTERS] = [
    Register::FaultIP as usize,
    Register::ra as usize,
    Register::sp as usize,
    Register::gp as usize,
    Register::s0 as usize,
    Register::s1 as usize,
    Register::s2 as usize,
    Register::s3 as usize,
    Register::s4 as usize,
    Register::s5 as usize,
    Register::s6 as usize,
    Register::s7 as usize,
    Register::s8 as usize,
    Register::s9 as usize,
    Register::s10 as usize,
    Register::s11 as usize,
];

pub const GP_REGISTERS: [usize; NUM_GP_REGISTERS] = [
    Register::a0 as usize,
    Register::a1 as usize,
    Register::a2 as usize,
    Register::a3 as usize,
    Register::a4 as usize,
    Register::a5 as usize,
    Register::a6 as usize,
    Register::a7 as usize,
    Register::t0 as usize,
    Register::t1 as usize,
    Register::t2 as usize,
    Register::t3 as usize,
    Register::t4 as usize,
    Register::t5 as usize,
    Register::t6 as usize,
    Register::tp as usize,
];

#[derive(Default, Clone, Copy, Debug)]
pub struct MessageInfo {
    pub words: [usize; 1],
}

impl MessageInfo {
    pub fn new(label: InvocationLabel, caps_unwrapped: usize, extra_caps: usize, length: usize) -> Self {
        let mut msg = MessageInfo { words: [0; 1]};
        msg.words[0] = 0
            | ((label as usize) & 0xfffffffffffff) << 12
            | (caps_unwrapped & 0x7) << 9
            | (extra_caps & 0x3) << 7
            | (length & 0x7f) << 0;
        msg
    }

    pub fn from_word(word: usize) -> Self {
        let mut msg = MessageInfo { words: [0; 1]};
        msg.words[0] = word;
        msg
    }

    pub fn to_word(&self) -> usize {
        self.words[0]
    }

    pub fn get_label(&self) -> usize {
        Self::sign_extend((self.words[0] & 0xfffffffffffff000) >> 12, 0x0)
    }

    pub fn get_extra_caps(&self) -> usize {
        Self::sign_extend((self.words[0] & 0x180) >> 7, 0x0)
    }

    pub fn get_length(&self) -> usize {
        Self::sign_extend((self.words[0] & 0x7f) >> 0, 0x0)
    }

    fn sign_extend(ret: usize, sign: usize) -> usize {
        if ret & (1 << 63) != 0 {
            return ret | sign;
        }
        ret
    }
}

pub enum InvocationLabel {
    InvalidInvocation = 0,
    UntypedRetype = 1,
    TCBReadRegisters = 2,
    TCBWriteRegisters = 3,
    TCBCopyRegisters = 4,
    TCBConfigure = 5,
    TCBSetPriority = 6,
    TCBSetMCPriority = 7,
    TCBSetSchedParams = 8,
    TCBSetIPCBuffer = 9,
    TCBSetSpace = 10,
    TCBSuspend = 11,
    TCBResume = 12,
    TCBBindNotification = 13,
    TCBUnbindNotification = 14,
    TCBSetTLSBase = 15,
    CNodeRevoke = 16,
    CNodeDelete = 17,
    CNodeCancelBadgedSends = 18,
    CNodeCopy = 19,
    CNodeMint = 20,
    CNodeMove = 21,
    CNodeMutate = 22,
    CNodeRotate = 23,
    CNodeSaveCaller = 24,
    IRQIssueIRQHandler = 25,
    IRQAckIRQ = 26,
    IRQSetIRQHandler = 27,
    IRQClearIRQHandler = 28,
    DomainSetSet = 29,
    PageTableMap = 30,
    PageTableUnmap = 31,
    PageMap = 32,
    PageUnmap = 33,
    PageGetAddress = 34,
    nInvocationLabels = 35,
}

impl InvocationLabel {
    pub fn from_usize(label: usize) -> Self {
        assert!(label >= InvocationLabel::InvalidInvocation as usize && label < InvocationLabel::nInvocationLabels as usize);
        unsafe {
            core::mem::transmute::<u8, InvocationLabel>(label as u8)
        }
    }
}



