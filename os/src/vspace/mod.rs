use crate::{config::CONFIG_PT_LEVELS, utils::{get_lvl_page_size_bits, round_down, bit}};
use crate::mm::VirtRegion;


pub fn get_n_paging(it_v_reg: VirtRegion) -> usize {
    let mut ans: usize = 0;
    for i in 0..CONFIG_PT_LEVELS - 1 {
        let bits = get_lvl_page_size_bits(i);
        let start = round_down(it_v_reg.start, bits);
        let end = round_down(it_v_reg.end, bits);
        ans += (end - start) / bit(bits);
    }
    ans
}