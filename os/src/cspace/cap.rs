use crate::types::{Pptr, Vptr};
use crate::utils::{bit, bool2usize, sign_extend};
use log::debug;
use crate::config::MIN_UNTYPED_BITS;

#[derive(Clone, Copy)]
pub struct Cap {
    words: [usize; 2],
}

#[derive(Copy, Clone)]
pub struct MDBNode {
    words:[usize; 2]
}

#[derive(Copy, Clone)]
pub struct CapTableEntry {
    pub(crate) cap: Cap,
    pub(crate) mdb_node: MDBNode,
}

#[derive(Eq, PartialEq, Debug)]
pub enum CapTag {
    CapNullCap = 0,
    CapUntypedCap = 2,
    CapEndpointCap = 4,
    CapNotificationCap = 6,
    CapReplyCap = 8,
    CapCNodeCap = 10,
    CapThreadCap = 12,
    CapIrqControlCap = 14,
    CapIrqHandlerCap = 16,
    CapZombieCap = 18,
    CapDomainCap = 20,
    CapFrameCap = 1,
    CapPageTableCap = 3,
    CapASIDControlCap = 11,
    CapASIDPoolCap = 13
}

impl Cap {
    pub fn get_cap_type(&self) -> CapTag {
        unsafe {
            core::mem::transmute::<u8, CapTag>(((self.words[0] >> 59) & 0x1f) as u8)
        }
    }

    pub fn get_cap_pptr(&self) -> Pptr {
        match self.get_cap_type() {
            CapTag::CapUntypedCap => sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000),
            CapTag::CapCNodeCap => sign_extend((self.words[0] & 0x3fffffffff) << 1, 0xffffff8000000000),
            CapTag::CapPageTableCap => sign_extend((self.words[1] & 0xfffffffffe00) >> 9, 0xffffff8000000000),
            CapTag::CapASIDPoolCap => sign_extend((self.words[0] & 0x1fffffffff) << 2, 0xffffff8000000000),
            CapTag::CapThreadCap => sign_extend(self.words[0] & & 0x7fffffffff, 0xffffff8000000000),
            CapTag::CapFrameCap => self.get_frame_base_ptr(),
            _ => 0
        }
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

    pub fn get_frame_mapped_addr(&self) -> Vptr {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend(self.words[0] & 0x7fffffffff, 0xffffff8000000000)
    }

    pub fn get_frame_base_ptr(&self) -> Pptr {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        sign_extend((self.words[1] & 0xfffffffffe00) >> 9, 0xffffff8000000000)
    }

    pub fn get_ep_badge(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapEndpointCap);
        sign_extend(self.words[1] & 0xffffffffffffffff, 0x0)
    }

    pub fn get_nt_fn_badge(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapNotificationCap);
        sign_extend(self.words[1] & & 0xffffffffffffffff, 0x0)
    }

    pub fn get_untyped_cap_block_size(&self) -> usize {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        sign_extend(self.words[1] & 0x3f, 0x0)
    }

    pub fn set_untyped_cap_free_index(&mut self, size: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapUntypedCap);
        self.words[1] &= !0xfffffffffe000000;
        self.words[1] |= (size << 25) & 0xfffffffffe000000;
    }

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

    pub fn new_domain_cap() -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapDomainCap as usize & 0x1f) << 59;
        cap.words[1] = 0;
        cap
    }

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

    pub fn new_asid_pool_cap(cap_asid_base: usize, cap_asid_pool: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapASIDPoolCap as usize & 0x1f) << 59
            | (cap_asid_base & 0xffff) << 43
            | (cap_asid_pool & 0x7ffffffffc) >> 2;
        cap
    }

    pub fn new_asid_control_cap() -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapASIDControlCap as usize & 0x1f) << 59;

        cap
    }

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

    pub fn new_thread_cap(cap_tcb_ptr: usize) -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapThreadCap as usize &0x1f) << 59
            | cap_tcb_ptr & 0x7fffffffff;
        cap
    }

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

    pub fn new_null_cap() -> Self {
        let mut cap: Cap = Cap { words: [0, 0] };
        cap.words[0] = 0
            | (CapTag::CapNullCap as usize & 0x1f) << 59;
        cap
    }

    pub fn frame_cap_set_frame_mapped_address(&mut self, addr: usize) {
        assert_eq!(self.get_cap_type(), CapTag::CapFrameCap);
        self.words[0] &= !(0x7fffffffff);
        self.words[0] |= addr & 0x7fffffffff;
    }
}

impl MDBNode {
    pub fn new(mdb_next: usize, mdb_revocable: bool, mdb_first_badged: bool, mdb_prev: usize) -> Self {
        let mut mdb_node = MDBNode {words: [0, 0]};
        mdb_node.words[0] = 0
            | mdb_prev << 0;
        mdb_node.words[1] = 0
            | (mdb_next & 0x7ffffffffc) >> 0
            | (bool2usize(mdb_revocable) & 0x1) << 1
            | (bool2usize(mdb_first_badged) & 0x1) << 0;
        mdb_node
    }
    pub fn null_mdbnode() -> Self {
        Self::new(0, false, false, 0)
    }

    pub fn set_mdb_prev(&mut self, v64: usize) {
        self.words[0] &= !0xffffffffffffffff;
        self.words[0] |= v64;
    }

    pub fn set_mdb_revocable(&mut self, mdb_revocable: bool) {
        self.words[1] &= !(0x2 as usize);
        self.words[1] |= (bool2usize(mdb_revocable) << 1) & (0x2 as usize);
    }

    pub fn set_mdb_first_badged(&mut self, mdb_first_badged: bool) {
        self.words[1] &= !(0x1 as usize);
        self.words[1] |= (bool2usize(mdb_first_badged) << 0) & (0x1 as usize);
    }

    pub fn set_mdb_next(&mut self, v64: usize) {
        self.words[1] &= !0x7ffffffffc;
        self.words[1] |= v64 & 0x7ffffffffc;
    }

    pub fn get_mdb_next(&self) -> usize {
        sign_extend(self.words[1] & 0x7ffffffffc, 0xffffff8000000000)
    }
}


pub fn is_cap_revocable(derived_cap: Cap, src_cap: Cap) -> bool {
    return match derived_cap.get_cap_type() {
        CapTag::CapUntypedCap => {
            true
        }
        CapTag::CapEndpointCap => {
            derived_cap.get_ep_badge() != src_cap.get_ep_badge()
        }
        CapTag::CapNotificationCap => {
            derived_cap.get_nt_fn_badge() != src_cap.get_nt_fn_badge()
        }
        CapTag::CapIrqHandlerCap => {
            src_cap.get_cap_type() == CapTag::CapIrqControlCap
        }
        _ => {
            false
        }
    }
}

