use common::{utils::sign_extend, types::Pptr};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn get_nt_fn_badge(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapNotificationCap);
        sign_extend(self.words[1] & & 0xffffffffffffffff, 0x0)
    }

    pub fn get_nt_fn_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapNotificationCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }

    pub fn set_nt_fn_badge(&mut self, v64: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapNotificationCap);
        self.words[1] &= !0xffffffffffffffff;
        self.words[1] |= v64 & 0xffffffffffffffff;
    }
}