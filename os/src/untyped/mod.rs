mod untyped;

use log::{debug, error, warn};
pub use untyped::UntypedDesc;
use crate::boot::{BootInfo, NDKS_BOOT};
use crate::config::{CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS, MAX_UNTYPED_BITS, MIN_UNTYPED_BITS, PPTR_BASE_OFFSET, WORD_BITS};
use crate::cspace::{Cap, CapTableEntry, CapTag, create_untyped_cap};
use crate::types::{Pptr, Region, SlotPos};
use crate::utils::{bit, bool2usize};

pub fn create_untyped_for_region(cnode_cap: Cap, is_device_mem: bool, reg: Region, first_slot: SlotPos) {
    let mut start = reg.start;
    while start < reg.end {

        let mut size_bits = WORD_BITS - 1 - ((reg.end - start).leading_zeros() as usize);
        if size_bits > MAX_UNTYPED_BITS {
            size_bits = MAX_UNTYPED_BITS;
        }
        if reg.start != 0 {
            let align_bits = start.trailing_zeros() as usize;
            if size_bits > align_bits {
                size_bits = align_bits;
            }
        }

        if size_bits >= MIN_UNTYPED_BITS {
            provide_untyped_cap(cnode_cap, is_device_mem, start, size_bits, first_slot);
        }

        start += bit(size_bits);
    }
}


pub fn provide_untyped_cap(cnode_cap: Cap, is_device_mem: bool, pptr: Pptr, size_bits: usize, first_slot: SlotPos) {
    let mut i = NDKS_BOOT.lock().slot_pos_cur - first_slot;
    if i < CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS {
        let boot_info = unsafe {
            &mut *(NDKS_BOOT.lock().boot_info_ptr as *mut BootInfo)
        };
        boot_info.untyped_list[i] = UntypedDesc::new(pptr - PPTR_BASE_OFFSET, size_bits, is_device_mem);
        let untyped_cap = create_untyped_cap(cnode_cap, NDKS_BOOT.lock().slot_pos_cur,
                                             max_free_index(size_bits), is_device_mem, size_bits, pptr);
        NDKS_BOOT.lock().slot_pos_cur += 1;
    } else {
        warn!("Kernel init: Too many untyped regions for boot info");
    }
}

pub fn set_untyped_cap_as_full(src_cap: Cap, new_cap: Cap, src_slot: &mut CapTableEntry) {
    if src_cap.get_cap_type() == CapTag::CapUntypedCap && new_cap.get_cap_type() == CapTag::CapUntypedCap {
        if src_cap.get_cap_pptr() == new_cap.get_cap_pptr() &&
            src_cap.get_untyped_cap_block_size() == new_cap.get_untyped_cap_block_size() {
            let ref_cap = &mut src_slot.cap;
            ref_cap.set_untyped_cap_free_index(max_free_index(src_cap.get_untyped_cap_block_size()));
        }
    }
}


#[inline]
fn max_free_index(size_bits: usize) -> usize {
    bit(size_bits - MIN_UNTYPED_BITS)
}
