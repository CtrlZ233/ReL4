use core::arch::asm;

pub const SYS_PUT_CHAR: usize = 0;

fn sysc_send_recv(sys: usize, dest: usize, info: usize, in_mr0: usize, in_mr1: usize, in_mr2: usize,
    in_mr3: usize) {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") dest,
            in("a1") info,
            in("a2") in_mr0,
            in("a3") in_mr1,
            in("a4") in_mr2,
            in("a5") in_mr3,
            in("a7") sys,
        );
    }
}

pub fn sys_put_char(v8: u8) {
    sysc_send_recv(SYS_PUT_CHAR, v8 as usize, 0, 0, 0, 0, 0);
}

