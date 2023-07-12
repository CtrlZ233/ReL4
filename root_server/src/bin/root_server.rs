#![no_std]
#![no_main]
#![feature(inline_const)]


extern crate root_server;

use core::{arch::asm, mem::size_of};
use root_server::BootInfo;
use user_lib::{println, thread::{sel4_tcb_suspend, sel4_tcb_configure, sel4_tcb_read_registers, sel4_tcb_write_registers, sel4_init_context_with_args, sel4_tcb_set_priority, sel4_tcb_resume}, untyped::sel4_untyped_retype, vspace::{sel4_page_map, sel4_page_table_map}};
use common::{types::{CNodeSlot, Cptr, IpcBuffer, CapRights}, object::ObjectType, register::UserContext};

static mut BOOT_INFO: usize = 0;
static mut IPC_BUFFER: usize = 0;


static CHILD_TCB_IPC_BUF_VADDR: usize = 0x100_0000;

static mut NEW_STACK: [u8; 4096] = [0u8; 4096];


fn set_env () {
    let mut reg_val: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) reg_val);
        BOOT_INFO = reg_val;
        IPC_BUFFER = get_boot_info().ipc_buf_ptr;
    }
}

pub fn get_boot_info() -> &'static mut BootInfo {
    unsafe {
        &mut *(BOOT_INFO as *mut BootInfo)
    }
}

#[no_mangle]
pub fn get_ipc_buffer() -> &'static mut IpcBuffer {
    unsafe {
        &mut *(IPC_BUFFER as *mut IpcBuffer)
    }
}

fn alloc_obj(t: ObjectType, user_obj_size: usize) -> Cptr {
    let untyped_size_bits = t.get_size(user_obj_size) as u8;
    let mut parent_untyped: Cptr = 0;
    let info = get_boot_info();
    let child_slot = info.empty.start;
    for i in 0..(info.untyped.end - info.untyped.start) {
        if info.untyped_list[i].size_bits >= untyped_size_bits && info.untyped_list[i].is_device == 0 {
            parent_untyped = info.untyped.start + i;
            break;
        }
    }

    assert_ne!(parent_untyped, 0);

    let error = sel4_untyped_retype(parent_untyped, t as usize, user_obj_size,
        CNodeSlot::SeL4CapInitThreadCNode as usize, 0, 0, child_slot, 1);

    assert_eq!(error, 0);
    info.empty.start += 1;
    info.untyped_list[parent_untyped - info.untyped.start].size_bits = 0;

    return child_slot;
}

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


#[no_mangle]
pub fn main() -> i32 {
    set_env();
    println!("hello root server!");
    
    // create tcb object
    let child_tcb = alloc_obj(ObjectType::TCBObject, 0);

    let ipc_buffer_pt = alloc_obj(ObjectType::RISCV_PageTableObject, 0);

    let new_ipc_buffer_frame = alloc_obj(ObjectType::RISCV_4KPage, 0);

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
    println!("bye root server!");
    0
}
