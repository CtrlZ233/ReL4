use common::{message::{InvocationLabel, MessageInfo}, register::UserContext};
use crate::{call_with_mrs, set_cap, set_mr, get_mr};
use common::types::{Cptr, Vptr};

pub fn sel4_tcb_suspend(service: Cptr) -> usize {
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

pub fn sel4_tcb_configure(service: Cptr, fault_ep: Cptr, cspace_root: Cptr, cspace_root_data: usize,
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


pub fn sel4_tcb_set_priority(service: Cptr, authority: Cptr, priority: usize) -> isize {
    let tag = MessageInfo::new(InvocationLabel::TCBSetPriority, 0, 1, 1);
    set_cap(0, authority);
    let mut mr0 =  priority;
    let mut mr1 = 0;
    let mut mr2 = 0;
    let mut mr3 = 0;

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

pub fn sel4_tcb_read_registers(service: Cptr, suspend_source: usize, arch_flags: u8, count: usize,
    regs: &mut UserContext) -> isize {

    let tag = MessageInfo::new(InvocationLabel::TCBReadRegisters, 0, 0, 2);
    let mut mr0 =  (suspend_source & 0x1) | ((arch_flags as usize & 0xff) << 8);
    let mut mr1 = count;
    let mut mr2 = 0;
    let mut mr3 = 0;

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.get_label();
    if result != 0 {
        set_mr(0, mr0);
        set_mr(1, mr1);
        set_mr(2, mr2);
        set_mr(3, mr3);
        return -1;
    }

    (*regs).pc = mr0;
    (*regs).ra = mr1;
    (*regs).sp = mr2;
    (*regs).gp = mr3;

    (*regs).s0 = get_mr(4);
    (*regs).s1 = get_mr(5);
    (*regs).s2 = get_mr(6);
    (*regs).s3 = get_mr(7);
    (*regs).s4 = get_mr(8);
    (*regs).s5 = get_mr(9);
    (*regs).s6 = get_mr(10);
    (*regs).s7 = get_mr(11);
    (*regs).s8 = get_mr(12);
    (*regs).s9 = get_mr(13);
    (*regs).s10 = get_mr(14);
    (*regs).s11 = get_mr(15);
    (*regs).a0 = get_mr(16);
    (*regs).a1 = get_mr(17);
    (*regs).a2 = get_mr(18);
    (*regs).a3 = get_mr(19);
    (*regs).a4 = get_mr(20);
    (*regs).a5 = get_mr(21);
    (*regs).a6 = get_mr(22);
    (*regs).a7 = get_mr(23);
    (*regs).t0 = get_mr(24);
    (*regs).t1 = get_mr(25);
    (*regs).t2 = get_mr(26);
    (*regs).t3 = get_mr(27);
    (*regs).t4 = get_mr(28);
    (*regs).t5 = get_mr(29);
    (*regs).t6 = get_mr(30);
    (*regs).tp = get_mr(31);

    result as isize
    
}


pub fn sel4_tcb_write_registers(service: Cptr, resume_target: usize, arch_flags: u8, count: usize,
    regs: &UserContext) -> isize {

    let tag = MessageInfo::new(InvocationLabel::TCBWriteRegisters, 0, 0, 34);
    let mut mr0 =  (resume_target & 0x1) | ((arch_flags as usize & 0xff) << 8);
    let mut mr1 = count;
    let mut mr2 = regs.pc;
    let mut mr3 = regs.ra;
    set_mr(4, regs.sp);
    set_mr(5, regs.gp);
    set_mr(6, regs.s0);
    set_mr(7, regs.s1);
    set_mr(8, regs.s2);
    set_mr(9, regs.s3);
    set_mr(10, regs.s4);
    set_mr(11, regs.s5);
    set_mr(12, regs.s6);
    set_mr(13, regs.s7);
    set_mr(14, regs.s8);
    set_mr(15, regs.s9);
    set_mr(16, regs.s10);
    set_mr(17, regs.s11);
    set_mr(18, regs.a0);
    set_mr(19, regs.a1);
    set_mr(20, regs.a2);
    set_mr(21, regs.a3);
    set_mr(22, regs.a4);
    set_mr(23, regs.a5);
    set_mr(24, regs.a6);
    set_mr(25, regs.a7);
    set_mr(26, regs.t0);
    set_mr(27, regs.t1);
    set_mr(28, regs.t2);
    set_mr(29, regs.t3);
    set_mr(30, regs.t4);
    set_mr(31, regs.t5);
    set_mr(32, regs.t6);
    set_mr(33, regs.tp);

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

pub fn sel4_tcb_resume(service: Cptr) -> isize {
    let tag = MessageInfo::new(InvocationLabel::TCBResume, 0, 0, 0);
    let mut mr0 = 0;
    let mut mr1 = 0;
    let mut mr2 = 0;
    let mut mr3 = 0;

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

pub fn sel4_init_context_with_args(entry_point: usize, arg0: usize, arg1: usize, arg2: usize,
    local_stack: usize, context: &mut UserContext) {
    context.pc = entry_point;
    context.sp = local_stack;
    context.a0 = arg0;
    context.a1 = arg1;
    context.a2 = arg2;
}