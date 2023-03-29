use crate::config::{CONFIG_PT_LEVELS, PAGE_TABLE_INDEX_BITS, PAGE_BITS};

#[inline]
pub fn mask(n: usize) -> usize {
    bit(n) - 1
}

#[inline]
pub fn bit(n: usize) -> usize {
    1 << n
}

#[inline]
pub fn get_lvl_page_size_bits(n: usize) -> usize {
    PAGE_TABLE_INDEX_BITS * (CONFIG_PT_LEVELS - 1 - n) + PAGE_BITS
}

#[inline]
pub fn get_lvl_page_size(n: usize) -> usize {
    bit(get_lvl_page_size_bits(n))
}

#[inline]
pub fn get_pt_index(addr: usize, n: usize) -> usize {
    (addr >> get_lvl_page_size_bits(n)) & mask(PAGE_TABLE_INDEX_BITS)
}

#[inline]
pub fn is_aligned(n: usize, b: usize) -> bool {
    (n & mask(b)) == 0
}

#[inline]
pub fn round_down(n: usize, b: usize) -> usize {
    (n >> b) << b
}
