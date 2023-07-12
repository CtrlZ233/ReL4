use common::{message::{InvocationLabel, NUM_FRAME_REGISTERS, NUM_GP_REGISTERS, NUM_MSG_REGISTRES,
    MESSAGE_REGISTERS, FRAME_REGISTERS, GP_REGISTERS, MessageInfo},
    utils::{convert_to_mut_type_ref, hart_id, convert_to_type_ref}, 
            types::{Pptr, Cptr, Prio, IpcBuffer}, config::NULL_PRIO, register::{BADGE_REGISTER, MSG_INFO_REGISTER}};
use crate::{scheduler::{ThreadStateEnum::ThreadStateRestart, ThreadControlFlag, THREAD_CONTROL_UPDATE_SPACE,
        THREAD_CONTROL_UPDATE_IPC_BUFFER, THREAD_CONTROL_UPDATE_MCP, re_schedule, THREAD_CONTROL_UPDATE_PRIORITY,
        set_thread_state, get_current_mut_tcb}, cspace::{CapTableEntry, Cap, derive_cap, TCBCNodeIndex, 
            CapTag, cte_insert}, ipc::check_valid_ipcbuf, mm::is_valid_vtable_root};
use log::{debug, error};
use crate::scheduler::TCBCNode;

use crate::scheduler::{KS_CUR_THREAD, TCB};

use super::{CUR_EXTRA_CAPS, get_syscall_arg};

pub fn decode_tcb_invocation(inv_label: usize, length: usize, cap: Cap, slot: &mut CapTableEntry,
                             call: bool, buffer: Pptr) {
    match InvocationLabel::from_usize(inv_label) {
        InvocationLabel::TCBSuspend => {
            debug!("Suspend TCB");
            set_thread_state(ThreadStateRestart);
            let dest_tcb = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
            dest_tcb.suspend();
        }

        InvocationLabel::TCBConfigure => {
            decode_tcb_configure(cap, length, slot, buffer);
        }

        InvocationLabel::TCBReadRegisters => {
            decode_tcb_read_registers(cap, length, call, buffer);
        }

        InvocationLabel::TCBWriteRegisters => {
            decode_tcb_write_registers(cap, length, buffer);
        }

        InvocationLabel::TCBSetPriority => {
            decode_tcb_set_priority(cap, length, buffer);
        }

        InvocationLabel::TCBResume => {
            let target = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
            invoke_tcb_resume(target);
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
    debug!("[decode_tcb_configure] : {}, {}, {}, {:#x}", fault_ep, cspace_root_data, vspace_root_data, buffer_addr);
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
            error!("[kernel: decode_tcb_configure] derive_cap buffer cap failed : {:?}", buffer_cap.get_cap_type());
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
        error!("[kernel: decode_tcb_configure] derive_cap cspace cap failed: {:?}", cspace_cap.get_cap_type());
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
    // invoke_tcb_thread_control(target, Some(slot), fault_ep, NULL_PRIO, NULL_PRIO,
    //     cspace_cap, Some(cspace_slot), vspace_cap, Some(vspace_slot),
    //     buffer_addr, buffer_cap, buffer_slot, THREAD_CONTROL_UPDATE_SPACE | THREAD_CONTROL_UPDATE_IPC_BUFFER);

    if !invoke_tcb_thread_update_space(target, slot, fault_ep, cspace_cap, cspace_slot,
        vspace_cap, vspace_slot)
        || !invoke_tcb_thread_update_ipc_buffer(target, slot, buffer_addr, buffer_cap, buffer_slot.unwrap()) {
        
        error!("tcb_configure failed");
        return;
    }

}

fn decode_tcb_set_priority(cap: Cap, length: usize, buffer: Pptr) {
    if length < 1 && unsafe { CUR_EXTRA_CAPS[0] == 0 } {
        error!("TCB SetPriority: Truncated message.");
        return;
    }

    let new_prio = get_syscall_arg(0, buffer);
    let auth_cap = convert_to_mut_type_ref::<CapTableEntry>(unsafe { CUR_EXTRA_CAPS[0]} ).cap;
    if auth_cap.get_cap_type() != CapTag::CapThreadCap {
        error!("Set priority: authority cap not a TCB.");
        return;
    }

    let auth_tcb = convert_to_mut_type_ref::<TCB>(auth_cap.get_tcb_ptr());
    if !auth_tcb.check_prio(new_prio) {
        error!("Set priority: check_prio failed.");
        return;
    }

    set_thread_state(ThreadStateRestart);
    let target_tcb = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
    invoke_tcb_thread_update_priority(target_tcb, new_prio);
}

fn invoke_tcb_thread_update_space(target: &mut TCB, slot: &mut CapTableEntry, faultep: Cptr, croot_new_cap: Cap,
    croot_src_slot: &mut CapTableEntry, vroot_new_cap: Cap, vroot_src_slot: &mut CapTableEntry) -> bool {

    let tcap = Cap::new_thread_cap(target as *mut TCB as usize);
    let tcb_cnode_table = convert_to_mut_type_ref::<TCBCNode>(target.get_cnode_ptr_of_this());

    target.tcb_fault_handler = faultep;
    let croot_slot = &mut tcb_cnode_table[TCBCNodeIndex::TCBCTable as usize];
    if !croot_slot.delete(true) {
        error!("error to delete cspace cap");
        return false;
    }
    if croot_new_cap.same_obj_as(&croot_src_slot.cap) && tcap.same_obj_as(&slot.cap) {
        cte_insert(croot_new_cap, croot_src_slot, croot_slot);
    }

    let vroot_slot = &mut tcb_cnode_table[TCBCNodeIndex::TCBVTable as usize];
    if !vroot_slot.delete(true) {
        error!("error to delete vspace cap");
        return false;
    }
    if vroot_new_cap.same_obj_as(&vroot_src_slot.cap) && tcap.same_obj_as(&slot.cap) {
        cte_insert(vroot_new_cap, vroot_src_slot, vroot_slot);
    }
    return true;
}


fn invoke_tcb_thread_update_mcp(target: &mut TCB, mcp: usize) {
    target.tcb_mcp = mcp;
}

fn invoke_tcb_thread_update_ipc_buffer(target: &mut TCB, slot: &mut CapTableEntry, buffer_addr: usize,
    buffer_cap: Cap, buffer_src_slot: &mut CapTableEntry) -> bool {

    let tcap = Cap::new_thread_cap(target as *mut TCB as usize);
    let tcb_cnode_table = convert_to_mut_type_ref::<TCBCNode>(target.get_cnode_ptr_of_this());
    let buffer_slot = &mut tcb_cnode_table[TCBCNodeIndex::TCBBuffer as usize];
    if !buffer_slot.delete(true) {
        error!("error to delete ipcbuffer");
        return false;
    }
    target.tcb_ipc_buffer = buffer_addr;
    if buffer_cap.same_obj_as(&buffer_src_slot.cap) && tcap.same_obj_as(&slot.cap) {
        cte_insert(buffer_cap, buffer_src_slot, buffer_slot);
    }

    if target as *mut TCB as usize == unsafe { KS_CUR_THREAD[hart_id()] } {
        re_schedule();
    }
    return true;
}

fn invoke_tcb_thread_update_priority(target: &mut TCB, prio: usize) {
    let tcap = Cap::new_thread_cap(target as *mut TCB as usize);
    target.set_priority(prio);
}


pub fn decode_tcb_read_registers(cap: Cap, length: usize, call: bool, buffer: Pptr) {
    if length < 2 {
        error!("TCB ReadRegisters: Truncated message.");
        return;
    }

    let flags = get_syscall_arg(0, buffer);
    let count = get_syscall_arg(1, buffer);

    if count < 1 || count > NUM_FRAME_REGISTERS + NUM_GP_REGISTERS {
        error!("TCB ReadRegisters: Attempted to read an invalid number of registers : {}", count);
        return;
    }

    let transfer_arch = 0;
    let thread = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
    if cap.get_tcb_ptr() == unsafe {KS_CUR_THREAD[hart_id()]} {
        error!("TCB ReadRegisters: Attempted to read our own registers.");
        return;
    }

    set_thread_state(ThreadStateRestart);
    invoke_tcb_read_registers(thread, flags != 0, count, transfer_arch, call)
}

fn invoke_tcb_read_registers(tcb: &mut TCB, suspend_source: bool, count: usize, _arch: usize, call: bool) {
    let current_tcb = get_current_mut_tcb();
    if suspend_source {
        tcb.suspend();
    }

    if call {
        current_tcb.set_register(BADGE_REGISTER, 0);
        let op_ipc_buffer = current_tcb.lookup_ipc_buffer(true);
        let mut i = 0;
        while i < count && i < NUM_FRAME_REGISTERS && i < NUM_MSG_REGISTRES {
            current_tcb.set_register(MESSAGE_REGISTERS[i], tcb.get_register(FRAME_REGISTERS[i]));
            i += 1;
        }

        if let Some(ipc_buffer) = op_ipc_buffer {
            while i < count && i < NUM_FRAME_REGISTERS {
                convert_to_mut_type_ref::<IpcBuffer>(ipc_buffer).msg[i] = tcb.get_register(FRAME_REGISTERS[i]);
                i += 1;
            }
        }

        let j = i;
        i = 0;

        if let Some(ipc_buffer) = op_ipc_buffer {
            while i < NUM_GP_REGISTERS && i + NUM_FRAME_REGISTERS < count {
                convert_to_mut_type_ref::<IpcBuffer>(ipc_buffer).msg[i + NUM_FRAME_REGISTERS]
                    = tcb.get_register(GP_REGISTERS[i]);
                    i += 1;
            }
        }
        current_tcb.set_register(MSG_INFO_REGISTER, 
            MessageInfo::new(InvocationLabel::InvalidInvocation, 0, 0, i + j).to_word());
    }

    current_tcb.set_thread_state(crate::scheduler::ThreadStateEnum::ThreadStateRunning);

}


fn decode_tcb_write_registers(cap: Cap, length: usize, buffer: Pptr) {
    if length < 2 {
        error!("TCB WriteRegisters: Truncated message.");
        return;
    }

    let flags = get_syscall_arg(0, buffer);
    
    let w = get_syscall_arg(1, buffer);
    if length < w + 2 {
        error!("TCB WriteRegisters: Message too short for requested write size {} / {}", length, w + 2);
        return;
    }

    let transfer_arch = 0;

    let thread = convert_to_mut_type_ref::<TCB>(cap.get_tcb_ptr());
    if cap.get_tcb_ptr() == unsafe {KS_CUR_THREAD[hart_id()]} {
        error!("TCB WriteRegisters: Attempted to read our own registers.");
        return;
    }

    set_thread_state(ThreadStateRestart);
    invoke_tcb_write_register(thread, flags != 0, w, transfer_arch, buffer);
}

fn invoke_tcb_write_register(dest: &mut TCB, resume_target: bool, count: usize, _arch: usize, buffer: Pptr) {
    let mut n = count;
    if count > NUM_FRAME_REGISTERS + NUM_GP_REGISTERS {
        n = NUM_FRAME_REGISTERS + NUM_GP_REGISTERS;
    }

    let mut i = 0;
    while i < NUM_FRAME_REGISTERS && i < n {
        dest.set_register(FRAME_REGISTERS[i], get_syscall_arg(i + 2, buffer));
        i += 1;
    }

    i = 0;
    while i < NUM_GP_REGISTERS && i + NUM_FRAME_REGISTERS < n {
        dest.set_register(GP_REGISTERS[i], get_syscall_arg(i + NUM_FRAME_REGISTERS + 2, buffer));
        i += 1;
    }

    let pc = dest.get_restart_pc();
    dest.set_next_pc(pc);

    if resume_target {
        dest.restart();
    }

    if dest as *mut TCB as usize == unsafe {KS_CUR_THREAD[hart_id()]} {
        re_schedule();
    }
}

fn invoke_tcb_resume(thread: &mut TCB) {
    thread.restart();
}