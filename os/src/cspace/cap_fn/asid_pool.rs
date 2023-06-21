use common::{utils::sign_extend, types::Pptr};

use crate::cspace::{Cap, CapTag};

impl Cap {
    pub fn get_asid_pool(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapASIDPoolCap);
        sign_extend((self.words[0] & 0x1fffffffff) << 2, 0xffffff8000000000)
    }

    pub fn new_asid_pool_cap(cap_asid_base: usize, cap_asid_pool: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapASIDPoolCap as usize & 0x1f) << 59
            | (cap_asid_base & 0xffff) << 43
            | (cap_asid_pool & 0x7ffffffffc) >> 2;
        cap
    }
}