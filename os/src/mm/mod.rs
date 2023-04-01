mod page_table;
mod asid;
use log::debug;
pub use page_table::PageTableEntry;
pub use asid::ASIDPool;

use riscv::register::satp;
use riscv::asm::sfence_vma_all;
use spin::Mutex;
use crate::types::VirtRegion;

use crate::utils::*;

use crate::config::{CONFIG_PT_LEVELS, PPTR_BASE, PPTR_TOP, PADDR_BASE, PPTR_BASE_OFFSET, ROOT_PAGE_TABLE_SIZE, KERNEL_ELF_BASE, KERNEL_ELF_PADDR_BASE, PAGE_BITS, PV_BASE_OFFSET, PAGE_TABLE_INDEX_BITS};
use crate::cspace::Cap;
use crate::mm::page_table::PTEFlags;
use crate::types::{Pptr, Vptr};

pub fn init() {
    map_kernel_window();
    activate_kernel_vspace();
}
#[no_mangle]
#[link_section = ".bss.root_pagetable"]
static mut KERNEL_ROOT_PAGE_TABLE: [u64; ROOT_PAGE_TABLE_SIZE] = [0; ROOT_PAGE_TABLE_SIZE];

#[no_mangle]
#[link_section = ".bss.l2_pagetable"]
static mut KERNEL_IMAGE_LEVEL2_PT: [u64; ROOT_PAGE_TABLE_SIZE] = [0; ROOT_PAGE_TABLE_SIZE];

static KERNEL_VSPACE_LOCK: Mutex<()> = Mutex::new(());

fn map_kernel_window() {
    unsafe {
        debug!("root pgtable addr: {:#x}, l2 pgtable addr: {:#x}",  &mut KERNEL_ROOT_PAGE_TABLE as *mut [u64; ROOT_PAGE_TABLE_SIZE] as usize
            , &mut KERNEL_IMAGE_LEVEL2_PT as *mut [u64; ROOT_PAGE_TABLE_SIZE] as usize)
    }
    
    assert!(CONFIG_PT_LEVELS > 1 && CONFIG_PT_LEVELS <= 4);
    let mut pptr = PPTR_BASE;
    let mut paddr = PADDR_BASE;
    let kernel_root_page_table = unsafe {
        & mut *(&mut KERNEL_ROOT_PAGE_TABLE as *mut [u64; ROOT_PAGE_TABLE_SIZE] as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };

    let kernel_image_level2_pt = unsafe {
        & mut *(&mut KERNEL_IMAGE_LEVEL2_PT as *mut [u64; ROOT_PAGE_TABLE_SIZE] as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };
    while pptr < PPTR_TOP {
        assert!(is_aligned(pptr, get_lvl_page_size_bits(0)));
        assert!(is_aligned(paddr, get_lvl_page_size_bits(0)));
        kernel_root_page_table[get_pt_index(pptr, 0)] = PageTableEntry::pte_next(paddr, true);
        // debug!("pptr: {:#x}, paddr: {:#x}", pptr, paddr);
        pptr += get_lvl_page_size(0);
        paddr += get_lvl_page_size(0);
    }

    assert!(pptr == PPTR_TOP);
    pptr = round_down(KERNEL_ELF_BASE, get_lvl_page_size_bits(0));
    paddr = round_down(KERNEL_ELF_PADDR_BASE, get_lvl_page_size_bits(0));
    let mut index: usize = 0;

    kernel_root_page_table[get_pt_index(KERNEL_ELF_PADDR_BASE + PPTR_BASE_OFFSET, 0)] = unsafe {
        PageTableEntry::pte_next(&mut KERNEL_IMAGE_LEVEL2_PT as *const [u64; ROOT_PAGE_TABLE_SIZE]  as usize - PV_BASE_OFFSET, false)
    };


    kernel_root_page_table[get_pt_index(pptr, 0)] = unsafe {
        PageTableEntry::pte_next(&mut KERNEL_IMAGE_LEVEL2_PT as *const [u64; ROOT_PAGE_TABLE_SIZE]  as usize - PV_BASE_OFFSET, false)
    };

    while pptr < PPTR_TOP + get_lvl_page_size(0) {
        kernel_image_level2_pt[index] = PageTableEntry::pte_next(paddr, true);
        index += 1;
        // debug!("pptr: {:#x}, paddr: {:#x}", pptr, paddr);
        pptr += get_lvl_page_size(1);
        paddr += get_lvl_page_size(1);
    }
}

pub fn activate_kernel_vspace() {
    let root_page_table_paddr = unsafe {
        &mut KERNEL_ROOT_PAGE_TABLE as *mut [u64; ROOT_PAGE_TABLE_SIZE] as usize - PV_BASE_OFFSET
    };
    debug!("root_page_table_paddr: {:#x}", root_page_table_paddr);
    unsafe {
        satp::set(satp::Mode::Sv39, 0, root_page_table_paddr >> PAGE_BITS);
        sfence_vma_all();
    }
}

pub fn get_n_paging(it_v_reg: VirtRegion) -> usize {
    let mut ans: usize = 0;
    for i in 0..CONFIG_PT_LEVELS - 1 {
        let bits = get_lvl_page_size_bits(i);
        let start = round_down(it_v_reg.start, bits);
        let end = round_down(it_v_reg.end, bits);
        ans += (end - start) / bit(bits);
    }
    ans
}

pub fn copy_global_mappings(newLvl1pt: &mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE]) {
    let _lock = KERNEL_VSPACE_LOCK.lock();
    let mut i = get_pt_index(PPTR_BASE, 0);
    let global_kernel_vspace = unsafe {
        & mut *(&mut KERNEL_IMAGE_LEVEL2_PT as *mut [u64; ROOT_PAGE_TABLE_SIZE] as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };

    while i < bit(PAGE_TABLE_INDEX_BITS) {
        newLvl1pt[i] = global_kernel_vspace[i];
        i += 1;
    }
}

pub fn map_it_pt_cap(vspace_cap: Cap, pt_cap: Cap) {
    let vptr = pt_cap.get_pt_mapped_addr();
    let lvl1pt = unsafe {
        &mut *(vspace_cap.get_cap_pptr() as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };
    let pt = pt_cap.get_cap_pptr();

    let (_, pte_pptr) = look_up_pt_slot(lvl1pt, vptr);
    let target_slot = unsafe {
        &mut *(pte_pptr as *mut PageTableEntry)
    };

    *target_slot = PageTableEntry::new((pt - PPTR_BASE_OFFSET) >> PAGE_BITS, 0, PTEFlags::V);
    unsafe {
        sfence_vma_all();
    }
}

pub fn map_frame_cap(vspace_cap: Cap, cap: Cap) {
    let lvl1pt = unsafe {
        &mut *(vspace_cap.get_cap_pptr() as *mut [PageTableEntry; ROOT_PAGE_TABLE_SIZE])
    };
    let frame_pptr = cap.get_cap_pptr();
    let frame_vptr = cap.get_frame_mapped_addr();

    let (pt_bits_left, pte_pptr) = look_up_pt_slot(lvl1pt, frame_vptr);
    assert_eq!(pt_bits_left, PAGE_BITS);
    let target_slot = unsafe {
        &mut *(pte_pptr as *mut PageTableEntry)
    };
    let flag = PTEFlags::D | PTEFlags::A | PTEFlags::U | PTEFlags::R | PTEFlags::W | PTEFlags::X | PTEFlags::V;
    *target_slot = PageTableEntry::new((frame_pptr - PPTR_BASE_OFFSET) >> PAGE_BITS, 0, flag);
    unsafe {
        sfence_vma_all();
    }
}

pub fn look_up_pt_slot(lvl1pt: &mut [PageTableEntry], vptr: Vptr) -> (usize, Pptr){
    let mut level = CONFIG_PT_LEVELS - 1;
    let mut pt = lvl1pt;
    let mut pt_bits_left = PAGE_TABLE_INDEX_BITS * level + PAGE_BITS;
    let mut pt_slot = &pt[(vptr >> pt_bits_left ) & mask(PAGE_TABLE_INDEX_BITS)];
    while pt_slot.is_pte_page_table() && 0 < level {
        level -= 1;
        pt_bits_left -= PAGE_TABLE_INDEX_BITS;
        pt = unsafe {
            &mut *(pt_slot.get_pptr_from_hw_pte() as *mut[PageTableEntry; ROOT_PAGE_TABLE_SIZE])
        };
        pt_slot = &pt[(vptr >> pt_bits_left ) & mask(PAGE_TABLE_INDEX_BITS)];
    }
    (pt_bits_left, pt_slot as *const PageTableEntry as Pptr)
}