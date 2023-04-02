use crate::types::Paddr;
use crate::utils::bool2usize;

const PADDING_LEN: usize = 8 - 2 * 1;
pub struct UntypedDesc {
    paddr: Paddr,
    size_bits: u8,
    is_device: u8,
    padding: [u8; PADDING_LEN],
}

impl UntypedDesc {
    pub fn new(paddr: Paddr, size_bits: usize, is_device: bool) -> Self {
        UntypedDesc {
            paddr,
            size_bits: size_bits as u8,
            is_device: bool2usize(is_device) as u8,
            padding: [0; PADDING_LEN]
        }
    }
}