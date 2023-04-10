use core::arch::asm;

pub const SYS_PUT_CHAR: usize = 0;
pub const SYS_CALL: usize = 1;
pub const SYS_SEND: usize = 2;

pub fn sysc_send_recv(sys: usize, dest: usize, &mut out_badge: usize, info: usize, &mut out_info: usize,
                      &mut in_out_mr0: usize, &mut in_out_mr1: usize, &mut in_out_mr2: usize, &mut in_out_mr3: usize) {
    let mut ret: isize;
    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") dest,
            in("a1") info,
            in("a2") *in_out_mr0,
            in("a3") *in_out_mr1,
            in("a4") *in_out_mr2,
            in("a5") *in_out_mr3,
            in("a7") sys,
        );
    }
    // TODO: out register
    // *out_info = info;
    // *out_badge = dest;
}

pub fn sys_put_char(v8: u8) {
    sysc_send_recv(SYS_PUT_CHAR, v8 as usize, &mut 0, 0,
                   &mut 0, &mut 0, &mut 0, &mut 0, &mut 0);
}

