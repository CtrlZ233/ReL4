#![no_std]

extern crate syscall;

use syscall::{MessageInfo, SYS_CALL};

mod console;
mod thread;

pub fn call_with_mrs(dest: usize, msg_info: MessageInfo, &mut mr0: usize, &mut mr1: usize, &mut mr2: usize, &mut mr3: usize)
    -> MessageInfo {
    let mut info = MessageInfo {words: [0; 1]};
    let mut msg0 = 0;
    let mut msg1 = 0;
    let mut msg2 = 0;
    let mut msg3 = 0;
    syscall::sysc_send_recv(SYS_CALL, dest, &mut dest, msg_info.words[0], &mut info.words[0],
                            &mut msg0, &mut msg1, &mut msg2, &mut msg3);

    *mr0 = msg0;
    *mr1 = msg1;
    *mr2 = msg2;
    *mr3 = msg3;
    info
}