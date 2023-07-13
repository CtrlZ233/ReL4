#[allow(non_camel_case_types)]
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

#[derive(Debug)]
pub struct UserContext {
    pub pc: usize,
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,

    pub s0: usize,
    pub s1: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,

    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,

    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
    
    pub tp: usize,
}

impl UserContext {
    pub fn new() -> Self {
        UserContext {
            pc: 0, ra: 0, sp: 0, gp: 0, s0: 0, s1: 0, s2: 0, s3: 0, s4: 0, s5: 0, s6: 0,
            s7: 0, s8: 0, s9: 0, s10: 0, s11: 0, a0: 0, a1: 0, a2: 0, a3: 0, a4: 0, a5: 0,
            a6: 0, a7: 0, t0: 0, t1: 0, t2: 0, t3: 0, t4: 0, t5: 0, t6: 0, tp: 0
        }
    }
}

pub const LR: usize = 0;
pub const SP: usize = 1;
pub const GP: usize = 2;
pub const TP: usize = 3;
pub const TLS_BASE: usize = TP;

pub const CAP_REGISTER: usize = 9;
pub const BADGE_REGISTER: usize = 9;
pub const MSG_INFO_REGISTER: usize = 10;

pub const SSTATUS_SPP: usize = 0x00000100;
pub const SSTATUS_FS: usize = 0x00006000;
pub const SSTATUS_SPIE: usize = 0x00000020;

