use common::{utils::sign_extend, types::Pptr};

use crate::cspace::{Cap, CapTag};


impl Cap {
    pub fn new_cnode_cap(cap_cnode_radix: usize, cap_cnode_guard_size: usize,
                         cap_cnode_guard: usize, cap_cnode_ptr: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (cap_cnode_radix & 0x3f) << 47
            | (cap_cnode_guard_size & 0x3f) << 53
            | (cap_cnode_ptr & 0x7ffffffffe) >> 1
            | (CapTag::CapCNodeCap as usize & 0x1f) << 59;
        cap.words[1] = 0
            | cap_cnode_guard << 0;
        cap
    }
    
    pub fn get_cnode_radix(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        sign_extend((self.words[0] & 0x1f800000000000) >> 47, 0x0)
    }

    pub fn get_cnode_guard_size(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        sign_extend((self.words[0] & 0x7e0000000000000) >> 53, 0x0)
    }

    pub fn get_cnode_guard(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        sign_extend(self.words[1] & 0xffffffffffffffff, 0x0)
    }

    pub fn get_cnode_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        sign_extend((self.words[0] & 0x3fffffffff) << 1, 0xffffff8000000000)
    }

    pub fn set_cnode_guard(&mut self, v64: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        self.words[1] &= !0xffffffffffffffff;
        self.words[1] |= v64 &0xffffffffffffffff;
    }

    pub fn set_cnode_guard_size(&mut self, v64: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapCNodeCap);
        self.words[0] &= !0x7e0000000000000;
        self.words[0] |= (v64 << 53) & 0x7e0000000000000;
    }
}