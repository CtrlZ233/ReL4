use common::types::{PTEPtr, ASIDSizeConstants};

pub struct ASIDPool {
    array: [PTEPtr; 1 << (ASIDSizeConstants::ASIDLowBits as usize)],
}

impl ASIDPool {
    pub fn write(&mut self, asid: usize, pte_ptr: PTEPtr) {
        self.array[asid] = pte_ptr;
    }
}