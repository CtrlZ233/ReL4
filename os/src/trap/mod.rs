use core::arch::{asm, global_asm};
use log::debug;
use crate::sbi;
use crate::scheduler::{KS_CUR_THREAD, TCB};
use common::utils::hart_id;
use riscv::register:: {
    stvec,
};
use crate::inner_syscall;
use riscv::register::stvec::TrapMode;
global_asm!(include_str!("trap.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            fn trap_entry();
        }
        stvec::write(trap_entry as usize, TrapMode::Direct);
    }
    // TODO: set timer
}

pub fn restore_user_context() {
    let cur_thread_reg_ptr = unsafe {
        let tcb = &*(KS_CUR_THREAD[hart_id()]as *const TCB);
        tcb.get_context_base_ptr()
    };
    // TODO: c_exit_hook
    unsafe {
        asm!(
            "mv t0, {0}",
            "ld ra, (0*8)(t0)",
            "ld sp, (1*8)(t0)",
            "ld gp, (2*8)(t0)",
            // skip tp
            // skip t0
            // no-op store conditional to clear monitor state
            // this may succeed in implementations with very large reservations, but the saved ra is dead
            "sc.d zero, zero, (t0)",
            "ld t2, (6*8)(t0)",
            "ld s0, (7*8)(t0)",
            "ld s1, (8*8)(t0)",
            "ld a0, (9*8)(t0)",
            "ld a1, (10*8)(t0)",
            "ld a2, (11*8)(t0)",
            "ld a3, (12*8)(t0)",
            "ld a4, (13*8)(t0)",
            "ld a5, (14*8)(t0)",
            "ld a6, (15*8)(t0)",
            "ld a7, (16*8)(t0)",
            "ld s2, (17*8)(t0)",
            "ld s3, (18*8)(t0)",
            "ld s4, (19*8)(t0)",
            "ld s5, (20*8)(t0)",
            "ld s6, (21*8)(t0)",
            "ld s7, (22*8)(t0)",
            "ld s8, (23*8)(t0)",
            "ld s9, (24*8)(t0)",
            "ld s10, (25*8)(t0)",
            "ld s11, (26*8)(t0)",
            "ld t3, (27*8)(t0)",
            "ld t4, (28*8)(t0)",
            "ld t5, (29*8)(t0)",
            "ld t6, (30*8)(t0)",
            // Get next restored tp
            "ld t1, (3*8)(t0)",
            // get restored tp
            "add tp, t1, x0",
            // get sepc
            "ld t1, (34*8)(t0)",
            "csrw sepc, t1",
            // Write back sscratch with cur_thread_reg to get it back on the next trap entry
            "csrw sscratch, t0",
            "ld t1, (32*8)(t0)",
            "csrw sstatus, t1",
            "ld t1, (5*8)(t0)",
            "ld t0, (4*8)(t0)",
            "sret",
            in(reg) cur_thread_reg_ptr,
        );
    }
    assert_eq!(1, 0);
}

#[no_mangle]
pub fn rust_handle_syscall(cptr: usize, msg_info: usize, syscall: isize) -> ! {

    // debug!("hello handle_syscall: cptr: {}, msg_info: {}, inner_syscall: {}", cptr, msg_info, inner_syscall);
    inner_syscall::slowpath(syscall);
    sbi::shutdown(false)
}

#[no_mangle]
pub fn rust_handle_interrupt() -> ! {
    debug!("hello handle_interrupt");
    sbi::shutdown(false)
}

#[no_mangle]
pub fn rust_handle_exception() -> ! {
    debug!("hello handle_exception");
    sbi::shutdown(false)
}
