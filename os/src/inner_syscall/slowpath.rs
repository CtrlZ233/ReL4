use crate::sbi;
use crate::scheduler::get_current_tcb;
use crate::scheduler::CAP_REGISTER;
use crate::trap::restore_user_context;
use log::error;
use crate::inner_syscall::syscall::handle_syscall;

use syscall::SYS_PUT_CHAR;

pub fn slowpath(syscall: isize) {
    match syscall {
        SYS_PUT_CHAR => {
            sbi::console_putchar(get_current_tcb().get_register(CAP_REGISTER));
        }
        _ => {
            error!("handle inner_syscall");
            handle_syscall(syscall);
        }
    }
    restore_user_context();
    
}