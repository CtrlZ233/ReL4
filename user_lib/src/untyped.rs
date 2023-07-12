use common::types::Cptr;
use common::message::{InvocationLabel, MessageInfo};

use crate::{set_cap, set_mr, call_with_mrs};


pub fn sel4_untyped_retype(service: Cptr, dest_type: usize, size_bits: usize, root: Cptr, node_index: usize,
    node_depth: usize, node_offset: usize, num_objects: usize) -> isize {
    
    let tag = MessageInfo::new(InvocationLabel::UntypedRetype, 0, 1, 6);
    let mut mr0: usize = dest_type;
    let mut mr1: usize = size_bits;
    let mut mr2: usize = node_index;
    let mut mr3: usize = node_depth;
    set_cap(0, root);
    set_mr(4, node_offset);
    set_mr(5, num_objects);

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.get_label();
    if result != 0 {
        set_mr(0, mr0);
        set_mr(1, mr1);
        set_mr(2, mr2);
        set_mr(3, mr3);
        return -1;
    }
    
    result as isize
}

