use syscall::{InvocationLabel, MessageInfo};

pub fn tcb_suspend(service: usize) {
    let tag = MessageInfo::new(InvocationLabel::TCBSuspend, 0, 0, 0);

}