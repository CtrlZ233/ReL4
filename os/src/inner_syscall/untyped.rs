use common::{types::Pptr, message::InvocationLabel, object::{ObjectType, get_object_size}, config::*, utils::{convert_to_mut_type_ref, aligned_up, bit}};
use common::object::ObjectType::*;
use crate::{scheduler::{ThreadStateEnum::ThreadStateRestart, TCB}, cspace::{CNode, insert_new_cap}, mm::VmRights};
use crate::cspace::CapTag::CapCNodeCap;
use log::{debug, error};

use crate::{inner_syscall::{CUR_EXTRA_CAPS, get_syscall_arg}, cspace::{CapTableEntry, Cap, lookup_target_slot}, scheduler::set_thread_state};

pub fn decode_untyped_invocation(inv_label: usize, length: usize, slot: &mut CapTableEntry, cap: Cap,
    call: bool, buffer: Pptr) {
    
    assert!(inv_label == InvocationLabel::UntypedRetype as usize && length >= 6);
    let new_type = ObjectType::from_usize(get_syscall_arg(0, buffer));
    let user_obj_size = get_syscall_arg(1, buffer);
    let node_index = get_syscall_arg(2, buffer);
    let node_depth = get_syscall_arg(3, buffer);
    let node_offset = get_syscall_arg(4, buffer);
    let node_window = get_syscall_arg(5, buffer);
    debug!("new_type: {}, user_obj_size: {}, node_index: {}, node_depth: {}, node_offset: {}, node_window: {}", new_type as usize, user_obj_size, node_index, node_depth,
            node_offset, node_window);

    let root_slot = unsafe { CUR_EXTRA_CAPS[0] };
    if new_type >= ObjectTypeCount {
        error!("invaild new type: {}", new_type as usize);
        return;
    }

    let object_size = get_object_size(new_type, user_obj_size);
    if user_obj_size >= WORD_BITS || object_size > MAX_UNTYPED_BITS {
        error!("Untyped Retype: invalid object size: {}, {}", user_obj_size, object_size);
        return;
    }

    if new_type == CapTableObject && user_obj_size == 0 {
        error!("Untyped Retype: Requested CapTable size too small.");
        return;
    }

    if new_type == UntypedObject && user_obj_size < MIN_UNTYPED_BITS {
        error!("Untyped Retype: Requested UntypedItem size too small.");
        return;
    }

    let node_cap: Cap;

    if node_depth == 0 {
        node_cap = convert_to_mut_type_ref::<CapTableEntry>(root_slot).cap;
    } else {
        let root_cap = convert_to_mut_type_ref::<CapTableEntry>(root_slot).cap;
        match lookup_target_slot(root_cap, node_index, node_depth) {
            Some(slot) => {
                node_cap = unsafe {
                    (&*(slot)).cap
                }
            }
            _ => {
                error!("Untyped Retype: Invalid destination address.");
                return;
            }
        }
    }
    assert_eq!(node_cap.get_cap_type(), CapCNodeCap);
    let node_size = (1 as usize ) << node_cap.get_cnode_radix();
    assert!(node_offset < node_size - 1);

    assert!(node_window >= 1 && node_window <= CONFIG_RETYPE_FAN_OUT_LIMIT);
    assert!(node_window <= node_size - node_offset);

    let dest_cnode = convert_to_mut_type_ref::<CNode>(node_cap.get_cap_pptr());
    for i in 0..node_window {
        if !dest_cnode[node_offset + i].ensure_empty_slot() {
            error!("Untyped Retype: Slot {} in destination window non-empty.", node_offset + i);
            return;
        }
    }

    let mut free_index: usize = 0;
    let mut reset = true;
    if !slot.ensure_no_child() {
        free_index = cap.get_untyped_free_index();
        debug!("have child: get_untyped_free_index: {}", free_index);
        reset = false;
    }

    let free_ref = cap.get_untyped_ref(free_index);

    let untyped_free_bytes = bit(cap.get_untyped_cap_block_size()) - (free_index << MIN_UNTYPED_BITS);

    if (untyped_free_bytes >> object_size) < node_window {
        error!("Untyped Retype: Insufficient memory: {} : {} : {} : {} : {}",
                free_index << MIN_UNTYPED_BITS, cap.get_untyped_cap_block_size(), untyped_free_bytes, object_size, node_window);
        return;
    }

    let device_mem = cap.get_untyped_is_device();
    if device_mem && !new_type.is_frame_type() && new_type != UntypedObject {
        error!("Untyped Retype: Creating kernel objects with device untyped");
        return;
    }

    let aligned_free_ref = aligned_up(free_ref, object_size);

    set_thread_state(ThreadStateRestart);

    invoke_untyped_retype(slot, reset, aligned_free_ref, new_type, user_obj_size,
                          dest_cnode, node_offset, node_window, device_mem);
}


pub fn invoke_untyped_retype(src_slot: &mut CapTableEntry, reset: bool, retyped_base: Pptr, new_type: ObjectType, user_size: usize,
                             dest_cnode: &mut CNode, dest_offset: usize, dest_length: usize, device_mem: bool) {
    if reset {
        assert!(src_slot.reset_untyped_cap())
    }

    let totol_obj_size = dest_length << get_object_size(new_type, user_size);
    let free_ref = retyped_base + totol_obj_size;
    src_slot.cap.set_untyped_cap_free_index((free_ref - retyped_base) >> MIN_UNTYPED_BITS);
    create_new_objects(new_type, src_slot, dest_cnode, dest_offset, dest_length, retyped_base, user_size, device_mem);
}

pub fn create_new_objects(new_type: ObjectType, src_slot: &mut CapTableEntry, dest_cnode: &mut CNode,
                          dest_offset: usize, dest_length: usize, retype_base: Pptr, user_size: usize, device_mem: bool) {
    let obj_size = get_object_size(new_type, user_size);
    let next_free_area = retype_base;
    for i in 0..dest_length {
        let cap = create_object(new_type, next_free_area + (i << obj_size), user_size, device_mem);
        insert_new_cap(src_slot, &mut dest_cnode[dest_offset + i], cap);
    }

}

pub fn create_object(new_type: ObjectType,  region_base: Pptr, user_size: usize, device_mem: bool) -> Cap {
    if new_type >= ObjectType::NonArchObjectTypeCount {
        return create_arch_object(new_type, region_base, user_size, device_mem);
    }
    debug!("region_base: {:#x}", region_base);
    match new_type {
        ObjectType::TCBObject => {
            let tcb = convert_to_mut_type_ref::<TCB>(region_base + TCB_OFFSET);
            tcb.init_context();
            tcb.tcb_domain = 0;
            return Cap::new_thread_cap(region_base + TCB_OFFSET);
        }
        _ => {

        }
    }
    Cap::new_null_cap()
}

pub fn create_arch_object(new_type: ObjectType,  region_base: Pptr, user_size: usize, device_mem: bool) -> Cap {
    match new_type {
        ObjectType::RISCV_4KPage => {
            Cap::new_frame_cap(0, region_base,  0,
                VmRights::VMReadWrite as usize, device_mem, 0)
        }

        ObjectType::RISCV_PageTableObject => {
            Cap::new_page_table_cap(0, region_base, false, 0)
        }
        _ => {
            error!("[create_arch_object] unsupported");
            Cap::new_null_cap()
        }
    }
    
}