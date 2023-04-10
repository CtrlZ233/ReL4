#![no_std]
#![no_main]
#![feature(inline_const)]


extern crate root_server;

use core::arch::asm;
use root_server::*;
use user_lib::{println, thread::tcb_suspend, CNodeSlot};


#[no_mangle]
pub fn main() -> i32 {
    println!("hello root server!");
    println!("===============");
    tcb_suspend(CNodeSlot::SeL4CapInitThreadTcb as usize);
    0
}
