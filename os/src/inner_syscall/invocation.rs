use common::register::{CAP_REGISTER, MSG_INFO_REGISTER};
use log::error;
use common::message::{MessageInfo, NUM_MSG_REGISTRES};
use common::config::{MSG_MAX_EXTRA_CAPS, SEL4_MSG_MAX_LEN};
use crate::cspace::{Cap, CapTableEntry, CapTag};
use crate::sbi::shutdown;
use crate::scheduler::{KS_CUR_THREAD, TCB};
use crate::scheduler::ThreadStateEnum::{ThreadStateRestart, ThreadStateRunning};
use common::types::Pptr;
use common::utils::{convert_to_mut_type_ref, hart_id};
use crate::inner_syscall::CUR_EXTRA_CAPS;

use super::tcb::decode_tcb_invocation;
use super::untyped::decode_untyped_invocation;
use super::vspace::{decode_frame_invocation, decode_page_table_invocation};

pub fn handle_invocation(is_call: bool , is_blocking: bool) {
    let thread = unsafe {
        convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
    };

    let info = MessageInfo::from_word(thread.get_register(MSG_INFO_REGISTER));
    let cptr = thread.get_register(CAP_REGISTER);
    match thread.lookup_cap_and_slot(cptr) {
        Some((cap, slot)) => {
            let buffer = thread.lookup_ipc_buffer(false);
            if !look_up_extra_caps(thread, buffer, info) {
                error!("look up extra caps failed");
                if is_blocking {
                    error!("need to handle fault");
                    shutdown(true);
                    // TODO: handle fault
                }
                return;
            }
            let mut length = info.get_length();
            if buffer.is_none() && length > NUM_MSG_REGISTRES {
                length = NUM_MSG_REGISTRES;
            }
            decode_invocation(info.get_label(), length, cptr, unsafe {&mut *(slot)}, cap,
                              is_blocking, is_call, buffer.unwrap());

            if thread.get_state() == ThreadStateRestart {
                if is_call {
                    thread.reply_from_kernel_success_empty();
                }
                thread.set_thread_state(ThreadStateRunning);
            }

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

fn decode_invocation(inv_label: usize, length: usize, cap_index: usize, slot: &mut CapTableEntry,
                         cap: Cap, block: bool, call: bool, buffer: Pptr) {
    match cap.get_cap_type() {
        CapTag::CapThreadCap => {
            decode_tcb_invocation(inv_label, length, cap, slot, call, buffer);
        }
        CapTag::CapUntypedCap => {
            decode_untyped_invocation(inv_label, length, slot, cap, call, buffer);
        }

        CapTag::CapFrameCap => {
            decode_frame_invocation(inv_label, length, slot, cap, call, buffer)
        }

        CapTag::CapPageTableCap =>  {
            decode_page_table_invocation(inv_label, length, slot, cap, buffer);
        }
        _ => {

        }
    }
}

fn look_up_extra_caps(tcb: &mut TCB, ipc_buffer: Option<Pptr>, msg: MessageInfo) -> bool {
    let length = msg.get_extra_caps();
    if length == 0 || ipc_buffer.is_none() {
        unsafe { CUR_EXTRA_CAPS[0] = 0; }
        return true;
    }
    let mut i: usize = 0;
    while i < length {
        let cptr = get_extra_cap_ptr(ipc_buffer.unwrap(), i);
        match tcb.lookup_slot(cptr) {
            Some(slot) => {
                unsafe { CUR_EXTRA_CAPS[i] = slot as usize; }
            }
            _ => {
                return false;
            }
        }
        i += 1;
    }
    if i < MSG_MAX_EXTRA_CAPS {
        unsafe {
            CUR_EXTRA_CAPS[i] = 0;
        }
    }
    true
}

fn get_extra_cap_ptr(buffer_ptr: Pptr, index: usize) -> usize {
    unsafe {
        *((buffer_ptr + (core::mem::size_of::<usize>() * (SEL4_MSG_MAX_LEN + 2 + index))) as *const usize)
    }
}

