#![no_std]
#![no_main]
#![feature(inline_const)]


extern crate root_server;

mod test;

use user_lib::println;

use crate::test::{utils::set_env, tcb_test::tcb_test};

#[no_mangle]
pub fn main() -> i32 {
    set_env();
    println!("hello root server!");
    tcb_test();
    println!("bye root server!");
    0
}
