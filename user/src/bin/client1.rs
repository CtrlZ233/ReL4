#![no_std]
#![no_main]
#![feature(inline_const)]

extern crate user_lib;
extern crate user;

use user_lib::println;

#[no_mangle]
pub fn main() -> i32 {
    println!("hello client1");
    0
}