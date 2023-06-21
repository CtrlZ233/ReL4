#![no_std]
#![feature(linkage)]

extern crate syscall;
extern crate common;
use common::{message::MessageInfo, types::{IpcBuffer, Cptr}};
use syscall::SYS_CALL;

pub mod console;
pub mod thread;
pub mod untyped;

pub fn call_with_mrs(dest: usize, msg_info: MessageInfo, mr0: &mut usize, mr1: &mut usize, mr2: &mut usize, mr3: &mut usize)
    -> MessageInfo {
    let mut local_dest = dest;
    let mut info = MessageInfo {words: [0; 1]};
    let mut msg0 = if msg_info.get_label() > 0 { *mr0 } else { 0 };
    let mut msg1 = if msg_info.get_label() > 1 { *mr1 } else { 0 };
    let mut msg2 = if msg_info.get_label() > 2 { *mr2 } else { 0 };
    let mut msg3 = if msg_info.get_label() > 3 { *mr3 } else { 0 };
    
    syscall::sysc_send_recv(SYS_CALL, dest, &mut local_dest, msg_info.words[0], &mut info.words[0],
                            &mut msg0, &mut msg1, &mut msg2, &mut msg3);
    *mr0 = msg0;
    *mr1 = msg1;
    *mr2 = msg2;
    *mr3 = msg3;
    info
}

#[linkage = "weak"]
#[no_mangle]
pub fn get_ipc_buffer() -> &'static mut IpcBuffer {
    panic!("invoke weak function!")
}

pub fn set_cap(index: usize, cptr: Cptr) {
    get_ipc_buffer().caps_or_badges[index] = cptr;
}

pub fn set_mr(index: usize, mr: usize) {
    get_ipc_buffer().msg[index] = mr;
}