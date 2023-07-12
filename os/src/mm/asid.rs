use common::{types::{PTEPtr, ASIDSizeConstants}, utils::{convert_to_mut_type_ref, mask}};
use crate::boot::KS_ASID_TABLE;
use log::{debug, error};

use super::PageTableEntry;
pub struct ASIDPool {
    array: [PTEPtr; 1 << (ASIDSizeConstants::ASIDLowBits as usize)],
}

impl ASIDPool {
    pub fn write(&mut self, asid: usize, pte_ptr: PTEPtr) {
        self.array[asid] = pte_ptr;
    }
}

pub fn find_vspace_for_asid(asid: usize) -> Option<&'static mut PageTableEntry> {
    let pool_ptr = unsafe {
        KS_ASID_TABLE.lock()[asid >> ASIDSizeConstants::ASIDLowBits as usize]
    };

    if pool_ptr == 0 {
        return None;
    }

    let asid_pool = convert_to_mut_type_ref::<ASIDPool>(pool_ptr);

    let vspace_root = asid_pool.array[asid & mask(ASIDSizeConstants::ASIDLowBits as usize)];
    if vspace_root == 0 {
        return None;
    }

    return Some(convert_to_mut_type_ref::<PageTableEntry>(vspace_root));
}