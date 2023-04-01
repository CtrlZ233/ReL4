mod domain_schedule;
mod tcb;
mod register;

use core::sync::atomic::AtomicUsize;
use lazy_static::*;
use log::debug;
use core::arch::asm;
use spin::Mutex;
use domain_schedule::DomainScheduler;

pub use tcb::{TCB, IdleTCB};

use crate::{config::{CPU_NUM, SEL4_IDLE_TCB_SLOT_SIZE, TCB_OFFSET, CONFIG_KERNEL_STACK_BITS}, types::Pptr};
lazy_static!{
    pub static ref KS_DOM_SCHEDULE: Mutex<[DomainScheduler; 1]> = Mutex::new([DomainScheduler{domain: 0, length: 60}]);
    pub static ref KS_DOM_SCHEDULE_IDX: AtomicUsize = AtomicUsize::new(0);
}

#[no_mangle]
#[link_section = ".kernel.idle_thread"]
static mut KS_IDLE_THREAD_TCB: [IdleTCB; CPU_NUM] = [IdleTCB {array: [0; SEL4_IDLE_TCB_SLOT_SIZE]}; CPU_NUM];

static mut KS_IDLE_THREAD: [Pptr; CPU_NUM] = [0; CPU_NUM];
static mut KERNEL_STACK: [[u8; 1 << CONFIG_KERNEL_STACK_BITS]; CPU_NUM] = [[0; 1 << CONFIG_KERNEL_STACK_BITS]; CPU_NUM];
static mut KS_CUR_THREAD: [Pptr; CPU_NUM] = [0; CPU_NUM];
static mut KS_SCHEDULER_ACTION: [Pptr; CPU_NUM] = [0; CPU_NUM];
const SCHEDULER_ACTION_RESUME_CURRENT_THREAD: usize = 0;

pub fn create_idle_thread() {
    debug!("sizeof TCB: {}", core::mem::size_of::<TCB>());
    let mut pptr: Pptr = 0;
    for i in 0..CPU_NUM {
        unsafe {
            pptr = &KS_IDLE_THREAD_TCB[i] as *const IdleTCB as Pptr;
            KS_IDLE_THREAD[i] = pptr + TCB_OFFSET;
            debug!("KS_IDLE_THREAD[i]: {:#x}", KS_IDLE_THREAD[i]);
            let tcb = &mut *(KS_IDLE_THREAD[i] as *mut TCB);
            tcb.configure_idle_thread();
        }
    }
}

pub fn idle_thread() {
    while true {
        unsafe {
            asm!("wfi");
        }
    }
}