#![no_std]

extern crate syscall;

use syscall::{MessageInfo, SYS_CALL};

pub mod console;
pub mod thread;

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

pub fn call_with_mrs(dest: usize, msg_info: MessageInfo, mr0: &mut usize, mr1: &mut usize, mr2: &mut usize, mr3: &mut usize)
    -> MessageInfo {
    let mut local_dest = dest;
    let mut info = MessageInfo {words: [0; 1]};
    let mut msg0 = 0;
    let mut msg1 = 0;
    let mut msg2 = 0;
    let mut msg3 = 0;
    syscall::sysc_send_recv(SYS_CALL, dest, &mut local_dest, msg_info.words[0], &mut info.words[0],
                            &mut msg0, &mut msg1, &mut msg2, &mut msg3);

    *mr0 = msg0;
    *mr1 = msg1;
    *mr2 = msg2;
    *mr3 = msg3;
    info
}