#![no_std]

mod syscall;
mod message;
pub use syscall::*;
pub use message::*;

#[inline]
fn sign_extend(ret: usize, sign: usize) -> usize {
    if ret & (1 << 63) != 0 {
        return ret | sign;
    }
    ret
}