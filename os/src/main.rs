//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.

#![deny(missing_docs)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(stdsimd)]
#![feature(core_intrinsics)]
#![allow(unused)]

extern crate bitflags;

extern crate syscall;
extern crate common;

use core::arch::global_asm;
use log::*;

#[macro_use]
mod console;
mod lang_items;
mod logging;
mod sbi;
mod mm;
mod boot;
mod root_server;
mod cspace;
mod scheduler;
mod ipc;
mod untyped;
mod trap;
mod inner_syscall;
mod object;
mod interrupt;


global_asm!(include_str!("entry.asm"));

/// clear BSS segment
pub fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

/// the rust entry-point of os
#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
        fn kernel_end();
    }

    clear_bss();
    logging::init();
    mm::init();
    boot::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);

    trap::init();
    debug!("kernel_end: {:#x}", kernel_end as usize);


    scheduler::schedule();
    scheduler::activate_thread();
    trap::restore_user_context();
    // CI autotest success: sbi::shutdown(false)
    // CI autotest failed : sbi::shutdown(true)
    sbi::shutdown(false)
}
