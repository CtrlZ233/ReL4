use common::{utils::{sign_extend, bool2usize}, types::{Pptr, Vptr}};

use super::super::cap::{Cap, CapTag};

impl Cap {
    pub fn new_frame_cap(cap_frame_mapped_asid: usize, cap_frame_base_ptr: usize,
                        cap_frame_size: usize, cap_frame_vm_right: usize,
                        cap_frame_is_device: bool, cap_frame_mapped_addr: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapFrameCap as usize & 0x1f) << 59
            | (cap_frame_size & 0x3) << 57
            | (cap_frame_vm_right & 0x3) << 55
            | (bool2usize(cap_frame_is_device) & 0x1) << 54
            | (cap_frame_mapped_addr & 0x7fffffffff) >> 0;

        cap.words[1] = 0
            | (cap_frame_mapped_asid & 0xffff) << 48
            | (cap_frame_base_ptr & 0x7fffffffff) << 9;
        cap
    }
    pub fn get_frame_mapped_addr(&self) -> Vptr {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }

    pub fn get_frame_base_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend((self.words[1] & 0xfffffffffe00) >> 9, 0xffffff8000000000)
    }

    pub fn get_frame_size(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend((self.words[0] & 0x600000000000000) >> 57, 0x0)
    }

    pub fn get_frame_is_device(&self) -> bool {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend((self.words[0] & 0x40000000000000) >> 54, 0x0) == 1
    }

    pub fn get_frame_vm_right(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend((self.words[0] & 0x180000000000000) >> 55, 0x0)
    }

    pub fn set_frame_mapped_address(&mut self, addr: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        self.words[0] &= !(0x7fffffffff);
        self.words[0] |= addr & 0x7fffffffff;
    }
}