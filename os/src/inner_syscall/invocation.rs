use log::error;
use syscall::{InvocationLabel, MessageInfo};
use crate::config::SEL4_MSG_MAX_LEN;
use crate::cspace::{Cap, CapTableEntry, CapTag};
use crate::scheduler::{CAP_REGISTER, KS_CUR_THREAD, MSG_INFO_REGISTER, TCB};
use crate::scheduler::ThreadStateEnum::ThreadStateRestart;
use crate::types::Pptr;
use crate::utils::hart_id;

pub fn handle_invocation(is_call: bool , is_blocking: bool) {
    let thread = unsafe {
        &mut *(KS_CUR_THREAD[hart_id()] as *mut TCB)
    };

    let info = MessageInfo::from_word(thread.get_register(MSG_INFO_REGISTER));
    let cptr = thread.get_register(CAP_REGISTER);
    match thread.lookup_cap_and_slot(cptr) {
        Some((cap, slot)) => {
            let buffer = thread.lookup_ipc_buffer(false).unwrap();
            let extra_caps = look_up_extra_caps(thread, buffer, info);
            let length = info.get_length();
            // TODO: judge the length
            decode_invocation(info.get_label(), length, cptr, slot, cap, is_blocking, is_call, buffer)

        }
        _ => {
            error!("[handle_invocation] look up slot failed!");
            if is_blocking {
                // TODO: handle fault
            }
        }
    }
}

pub fn decode_invocation(inv_label: usize, length: usize, cap_index: usize, slot: *mut CapTableEntry,
                         cap: Cap, block: bool, call: bool, buffer: Pptr) {
    match cap.get_cap_type() {
        CapTag::CapThreadCap => {
            decode_tcb_invocation(inv_label, length, cap, slot, call, buffer);
        }
        _ => {

        }
    }
}

pub fn decode_tcb_invocation(inv_label: usize, length: usize, cap: Cap, slot: *mut CapTableEntry,
                             call: bool, buffer: Pptr) {
    match InvocationLabel::from_usize(inv_label) {
        InvocationLabel::TCBSuspend => {
            let tcb = unsafe {
                &mut *(KS_CUR_THREAD[hart_id()] as *mut TCB)
            };
            tcb.set_thread_state(ThreadStateRestart);
            let dest_tcb = unsafe {
                &mut *(cap.get_tcb_ptr() as *mut TCB)
            };
            dest_tcb.suspend();
        }
        _ => {

        }
    }
}

pub fn look_up_extra_caps(tcb: &mut TCB, ipc_buffer: Pptr, msg: MessageInfo) -> Option<*mut CapTableEntry> {
    let length = msg.get_extra_caps();
    if length == 0 {
        return None;
    }
    let mut i: usize = 0;
    assert!(length <= 1);
    let cptr = get_extra_cap_ptr(ipc_buffer, i);
    tcb.lookup_slot(cptr)
}

pub fn get_extra_cap_ptr(buffer_ptr: Pptr, index: usize) -> usize {
    unsafe {
        *((buffer_ptr + (core::mem::size_of::<usize>() * (SEL4_MSG_MAX_LEN + 2 + index))) as *const usize)
    }
}