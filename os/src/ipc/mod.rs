use common::{types::Vptr, utils::is_aligned, config::SEL4_IPC_BUFFER_SIZE_BITS};

use crate::cspace::{Cap, CapTag};

pub fn check_valid_ipcbuf(vptr: Vptr, cap: Cap) -> bool {
    if cap.get_cap_type() != CapTag::CapFrameCap || cap.get_frame_is_device() || !is_aligned(vptr, SEL4_IPC_BUFFER_SIZE_BITS) {
        return false;
    }

    return true;
}