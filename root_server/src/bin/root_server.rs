#![no_std]
#![no_main]
#![feature(inline_const)]


extern crate root_server;

use core::arch::asm;
use root_server::*;

#[no_mangle]
pub fn main() -> i32 {
    unsafe {
        asm!(
        "add a1, a1, a2"
        );
    }
    0
}
