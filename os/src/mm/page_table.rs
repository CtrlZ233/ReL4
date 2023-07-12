use bitflags::*;
use common::config::{PAGE_BITS, PPTR_BASE_OFFSET};
use common::types::{Pptr, CapRights};
use common::utils::sign_extend;
use riscv::asm::sfence_vma_all;
bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: usize, sw: usize, flags: PTEFlags) -> Self {
        PageTableEntry {
            bits: ppn << 10 | (sw & 0x3) << 8 | flags.bits as usize,
        }
    }

    pub fn pte_next(paddr: usize, is_leaf: bool) -> Self {
        let ppn = paddr >> PAGE_BITS;
        let mut flag = PTEFlags::G | PTEFlags::V;
        if is_leaf {
            flag = flag | PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::D | PTEFlags::A;
        }

        Self::new(ppn, 0, flag)
    }

    pub fn make_user_pte(paddr: usize, executable: bool, vm_rights: VmRights) -> Self {
        let write = vm_rights.get_write();
        let read = vm_rights.get_read();
        if !read && !write && !executable {
            return Self::empty();
        }
        let mut flag = PTEFlags::D | PTEFlags::A | PTEFlags::V | PTEFlags::U;
        if read {
            flag |= PTEFlags::R;
        }

        if write {
            flag |= PTEFlags::W;
        }

        if executable {
            flag |= PTEFlags::X;
        }

        Self::new(paddr >> PAGE_BITS, 0, flag)
    }

    pub fn update(&mut self, pte: Self) {
        *self = pte;
        unsafe {
            sfence_vma_all();
        }
    }

    pub fn empty() -> Self {
        PageTableEntry { bits: 0 }
    }
    pub fn ppn(&self) -> usize {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn readable(&self) -> bool {
        (self.flags() & PTEFlags::R) != PTEFlags::empty()
    }
    pub fn writable(&self) -> bool {
        (self.flags() & PTEFlags::W) != PTEFlags::empty()
    }
    pub fn executable(&self) -> bool {
        (self.flags() & PTEFlags::X) != PTEFlags::empty()
    }

    pub fn is_pte_page_table(&self) -> bool {
        self.is_valid() && !(self.readable() || self.writable() || self.executable())
    }

    pub fn get_pptr_from_hw_pte(&self) -> Pptr {
        (self.ppn() << PAGE_BITS) + PPTR_BASE_OFFSET
    }
}


pub struct VMAttributes {
    word: [usize; 1],
}

impl VMAttributes {
    pub fn from_word(w: usize) -> Self {
        Self {
            word: [w],
        }
    }

    pub fn get_excute_never(&self) -> bool {
        sign_extend(self.word[0] & 0x1, 0x0) != 0
    }
}


#[derive(PartialEq, Eq, Clone, Copy)]
pub enum VmRights {
    VMKernelOnly = 1,
    VMReadOnly = 2,
    VMReadWrite = 3
}

impl VmRights {
    pub fn mask_vm_rights(&self, cap_rights_mask: CapRights) -> Self {
        if *self == VmRights::VMReadOnly && cap_rights_mask.get_allow_read() {
            return VmRights::VMReadOnly;
        }
        if *self == VmRights::VMReadWrite && cap_rights_mask.get_allow_read() {
            if !cap_rights_mask.get_allow_write() {
                return VmRights::VMReadOnly;
            } else {
                return VmRights::VMReadWrite;
            }
        }
        return VmRights::VMKernelOnly;
    }

    pub fn from_usize(value: usize) -> Self {
        unsafe {
            core::mem::transmute::<u8, VmRights>(value  as u8)
        }
    }

    pub fn get_write(&self) -> bool {
        *self == VmRights::VMReadWrite
    }

    pub fn get_read(&self) -> bool {
        *self != VmRights::VMKernelOnly
    }
}