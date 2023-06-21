use crate::cspace::{Cap, CapTag};

impl Cap {
    pub fn new_asid_control_cap() -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapASIDControlCap as usize & 0x1f) << 59;

        cap
    }
}