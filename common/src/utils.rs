use crate::{types::Paddr, config::PPTR_BASE_OFFSET};

use super::config::{CONFIG_PT_LEVELS, PAGE_TABLE_INDEX_BITS, PAGE_BITS, WORD_RADIX, L2_BITMAP_SIZE};

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
pub fn aligned_up(base_value: usize, alignment: usize) -> usize {
    (base_value + (bit(alignment) - 1)) & !mask(alignment)
}

#[inline]
pub fn round_down(n: usize, b: usize) -> usize {
    (n >> b) << b
}

#[inline]
pub fn round_up(n: usize, b: usize) ->usize {
    (((n - 1) >> b) + 1) << b
}

#[inline]
pub fn bool2usize(flag: bool) -> usize {
    if flag { 1 } else { 0 }
}

#[inline]
pub fn hart_id() -> usize {
    0
}

#[inline]
pub fn sign_extend(ret: usize, sign: usize) -> usize {
    if ret & (1 << 38) != 0 {
        return ret | sign;
    }
    ret
}

#[inline]
pub fn page_bits_for_size(page_size: usize) -> usize {
    // TODO: different page size
    assert!(page_size == 0);
    return PAGE_BITS;
}

#[inline]
pub fn prio_2_l1_index(prio: usize) -> usize {
    prio >> WORD_RADIX
}

#[inline]
pub fn l1_index_2_prio(index: usize) -> usize {
    index << WORD_RADIX
}

#[inline]
pub fn invert_l1_index(index: usize) -> usize {
    let invert = L2_BITMAP_SIZE - 1 - index;
    assert!(invert < L2_BITMAP_SIZE);
    invert
}
#[inline]
pub fn convert_to_mut_type_ref<T>(addr: usize) -> &'static mut T {
    assert_ne!(addr, 0);
    unsafe {
        &mut *(addr as *mut T)
    }
}

#[inline]
pub fn convert_to_type_ref<T>(addr: usize) -> &'static T {
    assert_ne!(addr, 0);
    unsafe {
        & *(addr as *mut T)
    }
}

#[inline]
pub fn addr_from_pptr(pptr: usize) -> Paddr {
    pptr - PPTR_BASE_OFFSET
}