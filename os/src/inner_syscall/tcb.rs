use common::{message::InvocationLabel, utils::{convert_to_mut_type_ref, hart_id, convert_to_type_ref}, types::{Pptr, Cptr, Prio}, config::NULL_PRIO};
use crate::{scheduler::{ThreadStateEnum::ThreadStateRestart, ThreadControlFlag, THREAD_CONTROL_UPDATE_SPACE, THREAD_CONTROL_UPDATE_IPC_BUFFER}, cspace::{CapTableEntry, Cap, derive_cap, CNode, TCBCNodeIndex, CapTag}, ipc::check_valid_ipcbuf, mm::{is_valid_vtable_root, PageTableEntry}};
use log::{debug, error};
use crate::scheduler::TCBCNode;

use crate::scheduler::{KS_CUR_THREAD, TCB};

use super::{CUR_EXTRA_CAPS, get_syscall_arg};

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

    let cspace_slot = unsafe { convert_to_mut_type_ref::<CapTableEntry>(CUR_EXTRA_CAPS[0]) };
    let mut cspace_cap = cspace_slot.cap;
    let vspace_slot = unsafe {  convert_to_mut_type_ref::<CapTableEntry>(CUR_EXTRA_CAPS[1]) };
    let mut vspace_cap = vspace_slot.cap;
    let mut buffer_slot  = Some(unsafe { convert_to_mut_type_ref::<CapTableEntry>(CUR_EXTRA_CAPS[2]) });
    let mut buffer_cap = buffer_slot.as_ref().unwrap().cap;

    if buffer_addr == 0 {
        buffer_slot = None;
    } else {
        let ret = derive_cap(buffer_slot.as_mut().unwrap(), buffer_cap);
        if !ret.0 {
            error!("[kernel: decode_tcb_configure] derive_cap buffer cap failed");
            return;
        }
        buffer_cap = ret.1;
        if !check_valid_ipcbuf(buffer_addr, buffer_cap) {
            error!("[kernel: decode_tcb_configure] ipc buffer is invalid");
        }
    }

    let tcb_cnode_table = convert_to_mut_type_ref::<TCBCNode>(convert_to_type_ref::<TCB>(cap.get_tcb_ptr()).get_cnode_ptr_of_this());
    let cnode = tcb_cnode_table[TCBCNodeIndex::TCBCTable as usize];
    let vspace_node = tcb_cnode_table[TCBCNodeIndex::TCBVTable as usize];
    if cnode.is_long_running_delete() || vspace_node.is_long_running_delete() {
        error!("[decode_tcb_configure]TCB Configure: CSpace or VSpace currently being deleted.");
    }

    if cspace_root_data != 0 {
        cspace_cap.update_cap_data(false, cspace_root_data);
    }

    let ret = derive_cap(cspace_slot, cspace_cap);
    if !ret.0 {
        error!("[kernel: decode_tcb_configure] derive_cap cspace cap failed");
        return;
    }

    cspace_cap = ret.1;
    if cspace_cap.get_cap_type() != CapTag::CapCNodeCap {
        error!("[kernel: decode_tcb_configure] CSpace cap is invalid");
        return;
    }

    if vspace_root_data != 0 {
        vspace_cap.update_cap_data(false, vspace_root_data);
    }

    let ret = derive_cap(vspace_slot, vspace_cap);
    if !ret.0 {
        error!("[kernel: decode_tcb_configure] derive_cap vspace cap failed");
        return;
    }

    vspace_cap = ret.1;
    if !is_valid_vtable_root(vspace_cap) {
        error!("[kernel: decode_tcb_configure] VSpace cap is invalid.");
        return;
    }

    let tcb = unsafe {
        convert_to_mut_type_ref::<TCB>(KS_CUR_THREAD[hart_id()])
    };
    tcb.set_thread_state(ThreadStateRestart);

    let target = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
    invoke_tcb_thread_control(target, slot, fault_ep, NULL_PRIO, NULL_PRIO,
        cspace_cap, cspace_slot, vspace_cap, vspace_slot,
        buffer_addr, buffer_cap, buffer_slot, THREAD_CONTROL_UPDATE_SPACE | THREAD_CONTROL_UPDATE_IPC_BUFFER);

}

// 
pub fn invoke_tcb_thread_control(target: &mut TCB, slot: &mut CapTableEntry, faultep: Cptr, mcp: Prio, priority: Prio, 
    croot_new_cap: Cap, croot_src_slot: &mut CapTableEntry, vroot_new_cap: Cap, vroot_src_slot: &mut CapTableEntry,
    buffer_addr: usize, buffer_cap: Cap, buffer_src_slot: Option<&mut CapTableEntry>, update_flags: ThreadControlFlag) {

    
}