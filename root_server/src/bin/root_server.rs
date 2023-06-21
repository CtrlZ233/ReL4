#![no_std]
#![no_main]
#![feature(inline_const)]


extern crate root_server;

use core::arch::asm;
use root_server::BootInfo;
use user_lib::{println, thread::tcb_suspend, untyped::untyped_retype};
use common::{types::{CNodeSlot, Cptr, IpcBuffer}, config::SEL4_TCB_BITS, object::ObjectType};

static mut BOOT_INFO: usize = 0;
static mut IPC_BUFFER: usize = 0;

fn set_env () {
    let mut reg_val: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) reg_val);
        BOOT_INFO = reg_val;
        IPC_BUFFER = get_boot_info().ipc_buf_ptr;
    }
}

pub fn get_boot_info() -> &'static mut BootInfo {
    unsafe {
        &mut *(BOOT_INFO as *mut BootInfo)
    }
}

#[no_mangle]
pub fn get_ipc_buffer() -> &'static mut IpcBuffer {
    unsafe {
        &mut *(IPC_BUFFER as *mut IpcBuffer)
    }
}

#[no_mangle]
pub fn main() -> i32 {
    set_env();
    println!("hello root server!");
    let info = get_boot_info();
    
    // create tcb object
    let untyped_size_bits = SEL4_TCB_BITS as u8;
    let mut parent_untyped: Cptr = 0;
    let child_tcb = info.empty.start;
    for i in 0..(info.untyped.end - info.untyped.start) {
        if info.untyped_list[i].size_bits >= untyped_size_bits && info.untyped_list[i].is_device == 0 {
            parent_untyped = info.untyped.start + i;
            break;
        }
    }

    assert_ne!(parent_untyped, 0);

    let error = untyped_retype(parent_untyped, ObjectType::TCBObject as usize, 0,
        CNodeSlot::SeL4CapInitThreadCNode as usize, 0, 0, child_tcb, 1);
    assert_eq!(error, 0);
    tcb_suspend(CNodeSlot::SeL4CapInitThreadTcb as usize);
    println!("bye root server!");
    0
}
