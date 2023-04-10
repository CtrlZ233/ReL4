use log::{error, debug};
use common::message::{InvocationLabel, MessageInfo, NUM_MSG_REGISTRES};
use common::config::SEL4_MSG_MAX_LEN;
use crate::cspace::{Cap, CapTableEntry, CapTag};
use crate::sbi::shutdown;
use crate::scheduler::{CAP_REGISTER, KS_CUR_THREAD, MSG_INFO_REGISTER, TCB};
use crate::scheduler::ThreadStateEnum::ThreadStateRestart;
use common::types::{Pptr, IpcBuffer};
use common::utils::hart_id;

pub fn handle_invocation(is_call: bool , is_blocking: bool) {
    let thread = unsafe {
        &mut *(KS_CUR_THREAD[hart_id()] as *mut TCB)
    };

    let info = MessageInfo::from_word(thread.get_register(MSG_INFO_REGISTER));
    let cptr = thread.get_register(CAP_REGISTER);
    match thread.lookup_cap_and_slot(cptr) {
        Some((cap, slot)) => {
            let buffer = thread.lookup_ipc_buffer(false);
            let extra_caps = look_up_extra_caps(thread, buffer, info);
            let mut length = info.get_length();
            if buffer.is_none() && length > NUM_MSG_REGISTRES {
                length = NUM_MSG_REGISTRES;
            }
            // TODO: judge the length
            decode_invocation(info.get_label(), length, cptr, slot, cap, is_blocking, is_call, buffer.unwrap())

        }
        _ => {
            error!("[handle_invocation] look up slot failed!");
            if is_blocking {
                error!("need to handle fault");
                shutdown(true);
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
        CapTag::CapUntypedCap => {
            decode_untyped_invocation(inv_label, length, slot, cap, call, buffer);
        }
        _ => {

        }
    }
}

pub fn decode_untyped_invocation(inv_label: usize, length: usize, slot: *mut CapTableEntry, cap: Cap,
    call: bool, buffer: Pptr) {
    
    assert!(inv_label == common::message::InvocationLabel::UntypedRetype as usize && length >= 6);
    let new_type = get_syscall_arg(0, buffer);
    let user_obj_size = get_syscall_arg(1, buffer);
    let node_index = get_syscall_arg(2, buffer);
    let node_depth = get_syscall_arg(3, buffer);
    let node_offset = get_syscall_arg(4, buffer);
    let node_window = get_syscall_arg(5, buffer);
    debug!("new_type: {}, user_obj_size: {}, node_index: {}, node_depth: {}, node_offset: {}, node_window: {}", new_type, user_obj_size, node_index, node_depth, 
            node_offset, node_window);
}

#[inline]
fn get_msg_register_by_arg_index(index: usize) -> usize {
    assert!(index < NUM_MSG_REGISTRES);
    match index {
        0 => crate::scheduler::Register::a2 as usize,
        1 => crate::scheduler::Register::a3 as usize,
        2 => crate::scheduler::Register::a4 as usize,
        3 => crate::scheduler::Register::a5 as usize,
        _ => {
            panic!("out of range")
        }
    }
}

pub fn get_syscall_arg(index: usize, ipc_buffer: Pptr) -> usize {
    if index < NUM_MSG_REGISTRES {
        let cur_tcb = unsafe {
            &*(KS_CUR_THREAD[hart_id()] as *const TCB)
        };
        return cur_tcb.get_register(get_msg_register_by_arg_index(index));
    }
    assert!(ipc_buffer != 0);
    unsafe {
        (&*(ipc_buffer as *mut IpcBuffer)).msg[index]
    }
}

pub fn decode_tcb_invocation(inv_label: usize, length: usize, cap: Cap, slot: *mut CapTableEntry,
                             call: bool, buffer: Pptr) {
    match InvocationLabel::from_usize(inv_label) {
        InvocationLabel::TCBSuspend => {
            debug!("Suspend TCB");
            let tcb = unsafe {
                &mut *(KS_CUR_THREAD[hart_id()] as *mut TCB)
            };
            tcb.set_thread_state(ThreadStateRestart);
            let dest_tcb = unsafe {
                &mut *(cap.get_tcb_ptr() as *mut TCB)
            };
            // debug!("dest_tcb: {:#x}, cur_tcb: {:#x}", tcb as *mut TCB as usize, dest_tcb as *mut TCB as usize);
            dest_tcb.suspend();
        }
        _ => {

        }
    }
}

pub fn look_up_extra_caps(tcb: &mut TCB, ipc_buffer: Option<Pptr>, msg: MessageInfo) -> Option<*mut CapTableEntry> {
    let length = msg.get_extra_caps();
    if length == 0 || ipc_buffer.is_none() {
        return None;
    }
    let mut i: usize = 0;
    assert!(length <= 1);
    let cptr = get_extra_cap_ptr(ipc_buffer.unwrap(), i);
    tcb.lookup_slot(cptr)
}

pub fn get_extra_cap_ptr(buffer_ptr: Pptr, index: usize) -> usize {
    unsafe {
        *((buffer_ptr + (core::mem::size_of::<usize>() * (SEL4_MSG_MAX_LEN + 2 + index))) as *const usize)
    }
}