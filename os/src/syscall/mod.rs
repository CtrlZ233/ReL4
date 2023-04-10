mod slowpath;
mod invocation;

pub use slowpath::slowpath;

pub const SYS_PUT_CHAR: usize = 0;
pub const SYS_CALL: usize = 1;