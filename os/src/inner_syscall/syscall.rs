use crate::{inner_syscall::invocation::handle_invocation, scheduler::{schedule, activate_thread}};
use log::debug;
use syscall::SYS_CALL;
pub fn handle_syscall(syscall: isize) {
    match syscall {
        SYS_CALL => {
            handle_invocation(true, true);
        }
        _ => {

        }
    }
    schedule();
    activate_thread();
    
}