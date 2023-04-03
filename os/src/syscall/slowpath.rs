
use super::SYS_PUT_CHAR;
use crate::sbi;
use crate::scheduler::get_current_tcb;
use crate::scheduler::CAP_REGISTER;
use crate::trap::restore_user_context;
use log::error;
pub fn slowpath(syscall: usize) {
    match syscall {
        SYS_PUT_CHAR => {
            sbi::console_putchar(get_current_tcb().get_register(CAP_REGISTER));
        }
        _ => {
            error!("unknown syscall");
        }
    }
    restore_user_context();
}