#![no_std]

mod syscall;
pub use syscall::*;

#[inline]
fn sign_extend(ret: usize, sign: usize) -> usize {
    if ret & (1 << 63) != 0 {
        return ret | sign;
    }
    ret
}