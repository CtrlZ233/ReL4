use crate::config::{PAGE_BITS, SEL4_WORD_BITS};
use crate::utils::{bit, clz32, round_up};

pub struct BootInfoHeader {
    id: usize,
    len: usize,
}

pub fn calculate_extra_bi_size_bits(extra_size: usize) -> usize {
    if extra_size == 0 {
        return 0;
    }
    let clzl_ret = clz32(round_up(extra_size, PAGE_BITS) as u32) + 32;
    // debug!("extra_size: {}, clzl_ret: {}", round_up(extra_size, PAGE_BITS), clzl_ret);
    let mut msb = SEL4_WORD_BITS - 1 - clzl_ret;
    if extra_size > bit(msb) {
        msb += 1;
    }
    msb
}