use crate::inner_syscall::invocation::handle_invocation;
use syscall::SYS_CALL;
pub fn handle_syscall(syscall: isize) {
    match syscall {
        SYS_CALL => {
            handle_invocation(true, true);
        }
        _ => {

        }
    }
}