use common::{utils::{sign_extend, bool2usize}, types::Pptr};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn new_reply_cap(cap_reply_can_grant: bool, cap_reply_master: bool, cap_tcb_ptr: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapReplyCap as usize & 0x1f) << 59
            | (bool2usize(cap_reply_can_grant) & 0x1) << 1
            | (bool2usize(cap_reply_master) & 0x1) << 0;
        cap.words[1] = 0
            | cap_tcb_ptr;
        cap
    }

    pub fn get_reply_tcb_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapReplyCap);
        sign_extend(self.words[1] & 0xffffffffffffffff, 0x0)
    }
}