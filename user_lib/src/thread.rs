use common::message::{InvocationLabel, MessageInfo};
use crate::{call_with_mrs, set_cap, set_mr};
use common::types::{Cptr, Vptr};

pub fn tcb_suspend(service: Cptr) -> usize {
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

pub fn tcb_configure(service: Cptr, fault_ep: Cptr, cspace_root: Cptr, cspace_root_data: usize,
                     vspace_root: Cptr, vspace_root_data: usize, buffer: Vptr, buffer_frame: Cptr) -> isize {
    let tag = MessageInfo::new(InvocationLabel::TCBConfigure, 0, 3, 4);
    set_cap(0, cspace_root);
    set_cap(1, vspace_root);
    set_cap(2, buffer_frame);
    let mut mr0 =  fault_ep;
    let mut mr1 = cspace_root_data;
    let mut mr2 = vspace_root_data;
    let mut mr3 = buffer;

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.get_label();
    if result != 0 {
        set_mr(0, mr0);
        set_mr(1, mr1);
        set_mr(2, mr2);
        set_mr(3, mr3);
        return -1;
    }
    result as isize
}