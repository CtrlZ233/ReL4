pub enum Register {
    ra = 0, sp = 1, gp = 2, tp = 3,

    t0 = 4, t1 = 5, t2 = 6, s0 = 7, s1 = 8,

    /* x10-x17 > a0-a7 */
    a0 = 9, a1 = 10, a2 = 11, a3 = 12,
    a4 = 13, a5 = 14, a6 = 15, a7 = 16, s2 = 17,
    s3 = 18, s4 = 19, s5 = 20, s6 = 21, s7 = 22,
    s8 = 23, s9 = 24, s10 = 25, s11 = 26,

    t3 = 27, t4 = 28, t5 = 29, t6 = 30,

    /* End of GP registers, the following are additional kernel-saved state. */
    SCAUSE = 31, SSTATUS = 32, FaultIP = 33, /* SEPC */
    NextIP = 34,
}

pub const LR: usize = 0;
pub const SP: usize = 1;
pub const GP: usize = 2;
pub const TP: usize = 3;
pub const TLS_BASE: usize = TP;

pub const CAP_REGISTER: usize = 9;
pub const BADGE_REGISTER: usize = 9;
pub const MSG_INFO_REGISTER: usize =10;

pub const SSTATUS_SPP: usize = 0x00000100;
pub const SSTATUS_FS: usize = 0x00006000;
pub const SSTATUS_SPIE: usize = 0x00000020;
