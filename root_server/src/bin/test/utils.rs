use core::arch::asm;

use common::{types::{CNodeSlot, Cptr, IpcBuffer}, object::ObjectType};
use root_server::BootInfo;
use user_lib::untyped::sel4_untyped_retype;

static mut BOOT_INFO: usize = 0;
static mut IPC_BUFFER: usize = 0;


pub fn set_env () {
    let mut reg_val: usize;
    unsafe {
        asm!("mv {}, a0", out(reg) reg_val);
        BOOT_INFO = reg_val;
        IPC_BUFFER = get_boot_info().ipc_buf_ptr;
    }
}

pub fn alloc_obj(t: ObjectType, user_obj_size: usize) -> Cptr {
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
