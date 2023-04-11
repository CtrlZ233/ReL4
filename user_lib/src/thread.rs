use syscall::{InvocationLabel, MessageInfo};
use crate::call_with_mrs;

pub fn tcb_suspend(service: usize) -> usize {
    let tag = MessageInfo::new(InvocationLabel::TCBSuspend, 0, 0, 0);
    let mut mr0: usize = 0;
    let mut mr1: usize = 0;
    let mut mr2: usize = 0;
    let mut mr3: usize = 0;

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);

    let result = output_tag.get_label();
    assert_eq!(result, 0);
    result
}