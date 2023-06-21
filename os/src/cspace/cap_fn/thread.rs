use common::{utils::sign_extend, types::Pptr};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn new_thread_cap(cap_tcb_ptr: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapThreadCap as usize &0x1f) << 59
            | cap_tcb_ptr & 0x7fffffffff;
        cap
    }
    
    pub fn get_tcb_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapThreadCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }
}