use crate::config::{SEL4_MSG_MAX_EXTRA_CAPS, SEL4_MSG_MAX_LEN};
use crate::types::Cptr;

pub struct IpcBuffer {
    tag: MessageInfo,
    msg: [usize; SEL4_MSG_MAX_LEN],
    user_data: usize,
    caps_or_badges: [usize; SEL4_MSG_MAX_EXTRA_CAPS],
    receive_cnode: Cptr,
    receive_index: Cptr,
    receive_depth: usize,
}

pub struct MessageInfo {
    words: [usize; 1],
}