use common::{utils::{sign_extend, bool2usize}, types::{Pptr}, config::MIN_UNTYPED_BITS};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn new_untyped_cap(cap_free_index: usize, cap_is_device: bool, cap_block_size: usize, cap_ptr: usize) -> Cap {
        let mut cap: Cap = Cap { words: [0, 0] };

        cap.words[0] = 0
            | (CapTag::CapUntypedCap as usize &0x1f) << 59
            | cap_ptr & 0x7fffffffff;

        cap.words[1] = 0
            | (cap_free_index & 0x7fffffffff) << 25
            | (bool2usize(cap_is_device) & 0x1) << 6
            | cap_block_size & 0x3f;
        cap
    }
    
    pub fn get_untyped_cap_block_size(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        sign_extend(self.words[1] & 0x3f, 0x0)
    }

    pub fn get_untyped_free_index(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        sign_extend((self.words[1] & 0xfffffffffe000000) >> 25, 0x0)
    }

    pub fn get_untyped_ref(&self, index: usize) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        self.get_untyped_ptr() + (index << MIN_UNTYPED_BITS)
    }

    pub fn get_untyped_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }

    pub fn get_untyped_is_device(&self) -> bool {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        sign_extend((self.words[1] & 0x40) >> 6, 0x0) == 1
    }

    pub fn set_untyped_cap_free_index(&mut self, size: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        self.words[1] &= !0xfffffffffe000000;
        self.words[1] |= (size << 25) & 0xfffffffffe000000;
    }
}