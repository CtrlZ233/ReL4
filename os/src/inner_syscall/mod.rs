mod slowpath;
mod invocation;
mod syscall;
mod untyped;
mod tcb;

use common::config::MSG_MAX_EXTRA_CAPS;
use common::message::NUM_MSG_REGISTRES;
use common::types::{Pptr, IpcBuffer};
use common::utils::{convert_to_mut_type_ref, hart_id};
pub use slowpath::slowpath;

use crate::scheduler::{KS_CUR_THREAD, TCB};


static mut CUR_EXTRA_CAPS: [Pptr; MSG_MAX_EXTRA_CAPS] = [0; MSG_MAX_EXTRA_CAPS];


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

fn get_syscall_arg(index: usize, ipc_buffer: Pptr) -> usize {
    if index < NUM_MSG_REGISTRES {
        let cur_tcb = unsafe {
            convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
        };
        return cur_tcb.get_register(get_msg_register_by_arg_index(index));
    }
    assert_ne!(ipc_buffer, 0);
    convert_to_mut_type_ref::<IpcBuffer>(ipc_buffer).msg[index]
}