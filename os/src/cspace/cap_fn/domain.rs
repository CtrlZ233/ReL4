use super::super::cap::{Cap, CapTag};


impl Cap {
    pub fn new_domain_cap() -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapDomainCap as usize & 0x1f) << 59;
        cap.words[1] = 0;
        cap
    }
}