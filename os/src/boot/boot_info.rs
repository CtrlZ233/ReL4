use crate::config::{PAGE_BITS, SEL4_WORD_BITS};
use crate::types::{NodeId, Paddr, Vptr, SlotRegion};
use crate::utils::{bit, clz32, round_up};

#[derive(Copy, Clone)]
pub struct BootInfoHeader {
    pub id: usize,
    pub len: usize,
}

pub enum BootInfoID {
    Sel4BootInfoHeaderPadding = 0,
    Sel4BootInfoHeaderX86Vbe = 1,
    Sel4BootInfoHeaderX86MbmMap = 2,
    Sel4BootInfoHeaderX86AcpiRsdp = 3,
    Sel4BootInfoHeaderX86Framebuffer = 4,
    Sel4BootInfoHeaderX86TscFreq = 5, /* frequency is in MHz */
    Sel4BootInfoHeaderFdt = 6, /* device tree */
    Sel4BootInfoHeaderNum,
}

pub struct UntypedDesc {
    paddr: Paddr,
    size_bits: u8,
    is_device: u8,
    padding: [u8; 8 - 2 * 1],
}

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
    pub untyped_list: UntypedDesc
}

pub fn calculate_extra_bi_size_bits(extra_size: usize) -> usize {
    if extra_size == 0 {
        return 0;
    }
    let clzl_ret = clz32(round_up(extra_size, PAGE_BITS) as u32) + 32;
    // debug!("extra_size: {}, clzl_ret: {}", round_up(extra_size, PAGE_BITS), clzl_ret);
    let mut msb = SEL4_WORD_BITS - 1 - clzl_ret;
    if extra_size > bit(msb) {
        msb += 1;
    }
    msb
}