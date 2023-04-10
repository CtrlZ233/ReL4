use super::sign_extend;

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

    pub fn get_label(&self) -> usize {
        sign_extend((self.words[0] & 0xfffffffffffff000) >> 12, 0x0)
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
    nInvocationLabels = 30,
}


