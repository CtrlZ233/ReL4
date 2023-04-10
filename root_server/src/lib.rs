#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::{asm, global_asm};

#[macro_use]
extern crate user_lib;

mod config;
mod lang_item;

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


