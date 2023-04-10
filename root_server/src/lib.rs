#![no_std]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::arch::{asm, global_asm};

#[macro_use]
extern crate user_lib;

extern crate common;

mod config;
mod lang_item;

use common::types::{NodeId, Vptr, SlotRegion, UntypedDesc};
use common::config::CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS;

#[derive(Debug)]
pub struct BootInfo {
    pub extra_len: usize,
    pub node_id: NodeId,
    pub num_nodes: usize,
    pub num_io_pt_levels: usize,
    pub ipc_buf_ptr: Vptr,
    pub empty: SlotRegion,
    pub shared_frames: SlotRegion,
    pub user_image_frames: SlotRegion,
    pub user_image_paging: SlotRegion,
    pub io_space_caps: SlotRegion,
    pub extra_bi_pages: SlotRegion,
    pub init_thread_cnode_size_bits: usize,
    pub init_thread_domain: usize,
    pub untyped: SlotRegion,
    pub untyped_list: [UntypedDesc; CONFIG_MAX_NUM_BOOT_INFO_UNTYPED_CAPS],
}

global_asm!(include_str!("entry.asm"));

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    unsafe {
        asm!(
        "add a1, a1, a0"
        );
    }
    panic!("Cannot find main!");
}


