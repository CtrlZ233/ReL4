use core::arch::asm;

pub const SYS_PUT_CHAR: isize = -9;
pub const SYS_CALL: isize = -1;
pub const SYS_SEND: isize = -3;

pub fn sysc_send_recv(sys: isize, dest: usize, out_badge: &mut usize, info: usize, out_info: &mut usize,
                      in_out_mr0: &mut usize, in_out_mr1: &mut usize, in_out_mr2: &mut usize, in_out_mr3: &mut usize) {
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
    unsafe {
        asm!("mv {}, a1", out(reg) *out_info);
        asm!("mv {}, a0", out(reg) *out_badge);
        asm!("mv {}, a2", out(reg) *in_out_mr0);
        asm!("mv {}, a3", out(reg) *in_out_mr1);
        asm!("mv {}, a4", out(reg) *in_out_mr2);
        asm!("mv {}, a5", out(reg) *in_out_mr3);
    }
}

pub fn sys_put_char(v8: u8) {
    sysc_send_recv(SYS_PUT_CHAR, v8 as usize, &mut 0, 0,
                   &mut 0, &mut 0, &mut 0, &mut 0, &mut 0);
}

