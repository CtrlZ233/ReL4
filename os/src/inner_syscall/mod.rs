mod slowpath;
mod invocation;
mod syscall;

use common::config::MSG_MAX_EXTRA_CAPS;
use common::types::Pptr;
pub use slowpath::slowpath;


static mut CUR_EXTRA_CAPS: [Pptr; MSG_MAX_EXTRA_CAPS] = [0; MSG_MAX_EXTRA_CAPS];