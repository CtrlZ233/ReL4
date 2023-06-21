use common::{utils::{sign_extend, bool2usize}, types::Vptr};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn new_page_table_cap(cap_pt_mapped_asid: usize, cap_pt_base_ptr: usize,
                            cap_pt_is_mapped: bool, cap_pt_mapped_address: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapPageTableCap as usize & 0x1f) << 59
            | (bool2usize(cap_pt_is_mapped) & 0x1) << 39
            | (cap_pt_mapped_address & 0x7fffffffff) >> 0;
        cap.words[1] = 0
            | (cap_pt_mapped_asid & 0xffff) << 48
            | (cap_pt_base_ptr & 0x7fffffffff) << 9;
        cap
    }

    pub fn get_pt_mapped_addr(&self) -> Vptr {
        assert_eq!(self.get_cap_type(), CapTag::CapPageTableCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }

    pub fn get_pt_mapped_asid(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapPageTableCap);
        sign_extend((self.words[1] & 0xffff000000000000) >> 48, 0x0)
    }

    pub fn get_pt_based_ptr(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapPageTableCap);
        sign_extend((self.words[1] & 0xfffffffffe00) >> 9, 0xffffff8000000000)
    }

    pub fn get_pt_is_mapped(&self) -> bool {
        assert_eq!(self.get_cap_type(), CapTag::CapPageTableCap);
         sign_extend(self.words[0] & 0x8000000000, 0x0) == 1
    }
}