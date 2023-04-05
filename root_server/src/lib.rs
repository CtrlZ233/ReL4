#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::{asm, global_asm};
use crate::config::USER_STACK_SIZE;

mod config;
mod lang_item;

static USER_STACK: [u8; USER_STACK_SIZE] = [0; USER_STACK_SIZE];

global_asm!(include_str!("entry.asm"));

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    unsafe {
        asm!(
        "add a1, a1, a0"
        );
    }
    panic!("Cannot find main!");
}


