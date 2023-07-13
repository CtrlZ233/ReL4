use core::mem::size_of;

use common::{object::ObjectType, types::{CNodeSlot, CapRights}, register::UserContext};
use user_lib::{vspace::{sel4_page_table_map, sel4_page_map}, thread::{sel4_tcb_configure, sel4_tcb_set_priority, sel4_tcb_read_registers, sel4_init_context_with_args, sel4_tcb_write_registers, sel4_tcb_resume, sel4_tcb_suspend}, println};

use super::utils::alloc_obj;

static CHILD_TCB_IPC_BUF_VADDR: usize = 0x100_0000;

static mut NEW_STACK: [u8; 4096] = [0u8; 4096];


fn test_mapped_ipc_buffer_frame() {
    unsafe {
        let x = CHILD_TCB_IPC_BUF_VADDR as *mut usize;
        *x = 1000;
        println!("x: {}", *x);
    }
}

fn new_thread(arg: usize) {
    println!("hello new thread: {}", arg);
    loop {}
}

pub fn tcb_test() {
    // create tcb object
    let child_tcb = alloc_obj(ObjectType::TCBObject, 0);

    let ipc_buffer_pt = alloc_obj(ObjectType::RiscvPageTableObject, 0);

    let new_ipc_buffer_frame = alloc_obj(ObjectType::Riscv4kpage, 0);

    let mut error = sel4_page_table_map(ipc_buffer_pt, CNodeSlot::SeL4CapInitThreadVspace as usize,
        CHILD_TCB_IPC_BUF_VADDR, common::types::VMAttributes::DefaultVMAttributes);

    assert_eq!(error, 0);

    error = sel4_page_map(new_ipc_buffer_frame, CNodeSlot::SeL4CapInitThreadVspace as usize,
        CHILD_TCB_IPC_BUF_VADDR, CapRights::new(1, 1, 1, 1),
        common::types::VMAttributes::DefaultVMAttributes);

    assert_eq!(error, 0);

    test_mapped_ipc_buffer_frame();

    error = sel4_tcb_configure(child_tcb, CNodeSlot::SeL4CapNull as usize,
        CNodeSlot::SeL4CapInitThreadCNode as usize, 0,
        CNodeSlot::SeL4CapInitThreadVspace as usize, 0,
        CHILD_TCB_IPC_BUF_VADDR, new_ipc_buffer_frame);
    assert_eq!(error, 0);

    error = sel4_tcb_set_priority(child_tcb, CNodeSlot::SeL4CapInitThreadTcb as usize, 254);
    assert_eq!(error, 0);

    let mut user_context = UserContext::new();
    error = sel4_tcb_read_registers(child_tcb, 0, 0,
        size_of::<UserContext>() / size_of::<usize>(), &mut user_context);
    assert_eq!(error, 0);

    let new_stack_top = unsafe {&mut NEW_STACK as *mut [u8; 4096]} as usize + 4096;

    sel4_init_context_with_args(new_thread as usize, 1024, 0, 0,
        new_stack_top, &mut user_context);
    println!("User Context: {:?}", user_context);
    // println!("UserContext count: {}", size_of::<UserContext>() / size_of::<usize>());
    error = sel4_tcb_write_registers(child_tcb, 0, 0,
        size_of::<UserContext>() / size_of::<usize>(), &user_context);
    assert_eq!(error, 0);

    let mut user_context2 = UserContext::new();
    error = sel4_tcb_read_registers(child_tcb, 0, 0,
        size_of::<UserContext>() / size_of::<usize>(), &mut user_context2);
    assert_eq!(error, 0);
    println!("User Context: {:?}", user_context2);
    

    error = sel4_tcb_resume(child_tcb);
    assert_eq!(error, 0);
    
    sel4_tcb_suspend(CNodeSlot::SeL4CapInitThreadTcb as usize);
}