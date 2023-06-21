use common::utils::sign_extend;

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn get_irq_handler(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapIrqHandlerCap);
        sign_extend(self.words[1] & 0xfff, 0x0)
    }
}