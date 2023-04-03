#![no_std]

mod syscall;
pub mod console;
pub use syscall::sys_put_char;
