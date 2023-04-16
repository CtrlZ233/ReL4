use log::{error, debug};
use common::message::{InvocationLabel, MessageInfo, NUM_MSG_REGISTRES};
use common::config::{CONFIG_RETYPE_FAN_OUT_LIMIT, MAX_UNTYPED_BITS, MIN_UNTYPED_BITS, MSG_MAX_EXTRA_CAPS, PAGE_BITS, SEL4_ENDPOINT_BITS, SEL4_MSG_MAX_LEN, SEL4_NOTIFICATION_BITS, SEL4_SLOT_BITS, SEL4_TCB_BITS, TCB_OFFSET, WORD_BITS};
use crate::cspace::{Cap, CapTableEntry, CapTag, CNode, insert_new_cap, lookup_target_slot};
use crate::sbi::shutdown;
use crate::scheduler::{CAP_REGISTER, KS_CUR_THREAD, MSG_INFO_REGISTER, set_thread_state, TCB};
use crate::scheduler::ThreadStateEnum::{ThreadStateRestart, ThreadStateRunning};
use common::types::{Pptr, IpcBuffer, ObjectType};
use common::types::ObjectType::{CapTableObject, ObjectTypeCount, UntypedObject};
use common::utils::{aligned_up, bit, convert_to_mut_type_ref, hart_id};
use crate::cspace::CapTag::CapCNodeCap;
use crate::inner_syscall::CUR_EXTRA_CAPS;

pub fn handle_invocation(is_call: bool , is_blocking: bool) {
    let thread = unsafe {
        convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
    };

    let info = MessageInfo::from_word(thread.get_register(MSG_INFO_REGISTER));
    let cptr = thread.get_register(CAP_REGISTER);
    match thread.lookup_cap_and_slot(cptr) {
        Some((cap, slot)) => {
            let buffer = thread.lookup_ipc_buffer(false);
            let ret = look_up_extra_caps(thread, buffer, info);
            if !ret {
                error!("look up extra caps failed");
                return;
            }
            let mut length = info.get_length();
            if buffer.is_none() && length > NUM_MSG_REGISTRES {
                length = NUM_MSG_REGISTRES;
            }
            decode_invocation(info.get_label(), length, cptr, unsafe {&mut *(slot)}, cap,
                              is_blocking, is_call, buffer.unwrap());

            if thread.get_state() == ThreadStateRestart {
                if is_call {
                    thread.reply_from_kernel_success_empty();
                }
                thread.set_thread_state(ThreadStateRunning);
            }

        }
        _ => {
            error!("[handle_invocation] look up slot failed!");
            if is_blocking {
                error!("need to handle fault");
                shutdown(true);
                // TODO: handle fault
            }
        }
    }
}

pub fn decode_invocation(inv_label: usize, length: usize, cap_index: usize, slot: &mut CapTableEntry,
                         cap: Cap, block: bool, call: bool, buffer: Pptr) {
    match cap.get_cap_type() {
        CapTag::CapThreadCap => {
            decode_tcb_invocation(inv_label, length, cap, slot, call, buffer);
        }
        CapTag::CapUntypedCap => {
            decode_untyped_invocation(inv_label, length, slot, cap, call, buffer);
        }
        _ => {

        }
    }
}

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

    let mut node_cap: Cap;

    if node_depth == 0 {
        node_cap = unsafe {
            convert_to_mut_type_ref::<CapTableEntry>(root_slot).cap
        };
    } else {
        let root_cap = unsafe {
            convert_to_mut_type_ref::<CapTableEntry>(root_slot).cap
        };
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
        reset = false;
    }

    let free_ref = cap.get_untyped_ref(free_index);

    let untyped_free_bytes = bit(cap.get_untyped_cap_block_size()) - (free_index << MIN_UNTYPED_BITS);

    if (untyped_free_bytes >> object_size) < node_window {
        error!("Untyped Retype: Insufficient memory");
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
    Cap::new_null_cap()
}

#[inline]
fn get_msg_register_by_arg_index(index: usize) -> usize {
    assert!(index < NUM_MSG_REGISTRES);
    match index {
        0 => crate::scheduler::Register::a2 as usize,
        1 => crate::scheduler::Register::a3 as usize,
        2 => crate::scheduler::Register::a4 as usize,
        3 => crate::scheduler::Register::a5 as usize,
        _ => {
            panic!("out of range")
        }
    }
}

pub fn get_syscall_arg(index: usize, ipc_buffer: Pptr) -> usize {
    if index < NUM_MSG_REGISTRES {
        let cur_tcb = unsafe {
            convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
        };
        return cur_tcb.get_register(get_msg_register_by_arg_index(index));
    }
    assert_ne!(ipc_buffer, 0);
    convert_to_mut_type_ref::<IpcBuffer>(ipc_buffer).msg[index]
}

pub fn decode_tcb_invocation(inv_label: usize, length: usize, cap: Cap, slot: &mut CapTableEntry,
                             call: bool, buffer: Pptr) {
    match InvocationLabel::from_usize(inv_label) {
        InvocationLabel::TCBSuspend => {
            debug!("Suspend TCB");
            let tcb = unsafe {
                convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
            };
            tcb.set_thread_state(ThreadStateRestart);
            let dest_tcb = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
            debug!("dest_tcb: {:#x}, cur_tcb: {:#x}", dest_tcb as *mut TCB as usize, tcb as *mut TCB as usize);
            dest_tcb.suspend();
        }

        InvocationLabel::TCBConfigure => {
            decode_tcb_configure(cap, length, slot, buffer);
        }
        _ => {

        }
    }
}

pub fn decode_tcb_configure(cap: Cap, length: usize, slot: &mut CapTableEntry, buffer: Pptr) {
    let tcb_configure_args: usize = 4;
    unsafe {
        if length < tcb_configure_args || CUR_EXTRA_CAPS[0] == 0 ||
            CUR_EXTRA_CAPS[1] == 0 || CUR_EXTRA_CAPS[2]== 0 {
            error!("TCB Configure: Truncated message.");
            return;
        }
    }
    let fault_ep = get_syscall_arg(0, buffer);
    let cspace_root_data = get_syscall_arg(1, buffer);
    let vspace_root_data = get_syscall_arg(2, buffer);
    let buffer_addr = get_syscall_arg(3, buffer);

    // let cspace_slot = unsafe { *(CUR_EXTRA_CAPS[0]) };
    // let cspace_cap = cspace_slot.cap;
    // let vspace_slot = unsafe {  *(CUR_EXTRA_CAPS[1]) };
    // let vspace_cap = vspace_slot.cap;
    // let buffer_slot = unsafe { *(CUR_EXTRA_CAPS[2]) };
    // let buffer_cap = buffer_slot.cap;


}

pub fn look_up_extra_caps(tcb: &mut TCB, ipc_buffer: Option<Pptr>, msg: MessageInfo) -> bool {
    let length = msg.get_extra_caps();
    if length == 0 || ipc_buffer.is_none() {
        unsafe { CUR_EXTRA_CAPS[0] = 0; }
        return true;
    }
    let mut i: usize = 0;
    while i < length {
        let cptr = get_extra_cap_ptr(ipc_buffer.unwrap(), i);
        match tcb.lookup_slot(cptr) {
            Some(slot) => {
                unsafe { CUR_EXTRA_CAPS[i] = slot as usize; }
            }
            _ => {
                return false;
            }
        }
        i += 1;
    }
    if i < MSG_MAX_EXTRA_CAPS {
        unsafe {
            CUR_EXTRA_CAPS[i] = 0;
        }
    }
    true
}

pub fn get_extra_cap_ptr(buffer_ptr: Pptr, index: usize) -> usize {
    unsafe {
        *((buffer_ptr + (core::mem::size_of::<usize>() * (SEL4_MSG_MAX_LEN + 2 + index))) as *const usize)
    }
}


pub fn get_object_size(t: ObjectType, user_object_size: usize) -> usize {
    match t {
        ObjectType::UntypedObject => user_object_size,
        ObjectType::TCBObject => SEL4_TCB_BITS,
        ObjectType::EndpointObject => SEL4_ENDPOINT_BITS,
        ObjectType::NotificationObject => SEL4_NOTIFICATION_BITS,
        ObjectType::CapTableObject => SEL4_SLOT_BITS + user_object_size,
        ObjectType::RISCV_4KPage | ObjectType::RISCV_PageTableObject => PAGE_BITS,
        _ => {
            error!("invalid object type: {}", t as usize);
            return 0;
        }
    }
}