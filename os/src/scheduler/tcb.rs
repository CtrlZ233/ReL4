use crate::{config::{CONTEXT_REGISTERS_NUM, SEL4_IDLE_TCB_SLOT_SIZE, CONFIG_KERNEL_STACK_BITS}, types::{Pptr, Dom, Prio, Cptr, Vptr}, utils::{bit, hart_id, sign_extend}};
use core::ops::{Index, IndexMut};
use super::{register::{Register, SSTATUS_SPP, SSTATUS_SPIE, SP}, idle_thread, KERNEL_STACK, KS_CUR_THREAD, KS_SCHEDULER_ACTION, SCHEDULER_ACTION_RESUME_CURRENT_THREAD};

use log::{error, debug};

#[derive(Default)]
pub struct TCB {
    context: RiscvContext,
    tcb_state: ThreadState,
    tcb_bound_notification: Pptr,
    tcb_fault: Fault,
    lookup_fault: LookUpFault,
    tcb_domain: Dom,
    tcb_mcp: Prio,
    tcb_priority: Prio,
    tcb_time_slice: usize,
    tcb_fault_handler: Cptr,
    tcb_ipc_buffer: Vptr,

    tcb_sched_next: Pptr,
    tcb_sched_prev: Pptr,

    tcb_ep_next: Pptr,
    tcb_ep_prev: Pptr,
}

impl TCB {
    pub fn configure_idle_thread(&mut self) {        
        self.set_register(Register::NextIP as usize, idle_thread as usize);
        
        self.set_register(Register::SSTATUS as usize, SSTATUS_SPIE | SSTATUS_SPP);
        
        let kernel_stack_ptr = unsafe {
            &KERNEL_STACK[0][0] as *const u8 as usize + bit(CONFIG_KERNEL_STACK_BITS)
        };
        self.set_register(SP, kernel_stack_ptr);
        self.set_thread_state(ThreadStateEnum::ThreadStateIdleThreadState);
    }

    pub fn set_thread_state(&mut self, ts: ThreadStateEnum) {
        self.tcb_state.set(ts as usize);
        self.schedule();
    }

    pub fn schedule(&mut self) {
        let self_ptr = self as *const TCB as Pptr;
        unsafe {
            if self_ptr == KS_CUR_THREAD[hart_id()] &&
                KS_SCHEDULER_ACTION[hart_id()] == SCHEDULER_ACTION_RESUME_CURRENT_THREAD &&
                !self.is_schedulable() {
                    // TODD: re_schedule
                    error!("todo: re_schedule");
                }
        }
    }

    pub fn is_schedulable(&self) -> bool {
        match self.get_state() {
            ThreadStateEnum::ThreadStateRunning | ThreadStateEnum::ThreadStateRestart => {
                return true;
            },
            _ => {
                return false
            }
        }
    }

    pub fn get_state(&self) -> ThreadStateEnum {
        unsafe {
            core::mem::transmute::<u8, ThreadStateEnum>(sign_extend(self.tcb_state.words[0] & 0xf, 0x0) as u8)
        }   
    }

    pub fn set_register(&mut self, reg: usize, w: usize) {
        self.context.registers[reg] = w;
    }
}

pub struct IdleTCB {
    pub array: [u8; SEL4_IDLE_TCB_SLOT_SIZE],
}

struct Array<T: Default + Copy, const N: usize>([T; N]);

impl<T: Default + Copy, const N: usize> Default for Array<T, N> {
    fn default() -> Self {
        let inner = [T::default(); N];
        Array(inner)
    }
}

impl<T: Default + Copy, const N: usize> Index<usize> for Array<T, N> {
    type Output = T;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T: Default + Copy, const N: usize> IndexMut<usize> for Array<T, N> {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}


#[derive(Default)]
struct RiscvContext {
    registers: Array<usize, CONTEXT_REGISTERS_NUM>,
}

#[derive(Default)]
struct ThreadState {
    words: Array<usize, 3>,
}

impl ThreadState {
    pub fn set(&mut self, ts: usize) {
        self.words[0] &= !0xf;
        self.words[1] |= (ts << 0) &0xf;

    }
}

#[derive(Default)]
struct Notification {
    words: Array<usize, 4>,
}

#[derive(Default)]
struct Fault {
    words: Array<usize, 2>,
}

#[derive(Default)]
struct LookUpFault {
    words: Array<usize, 2>,
}

pub enum ThreadStateEnum {
    ThreadStateInactive = 0,
    ThreadStateRunning = 1,
    ThreadStateRestart = 2,
    ThreadStateBlockedOnReceive = 3,
    ThreadStateBlockedOnSend = 4,
    ThreadStateBlockedOnReply = 5,
    ThreadStateBlockedOnNotification = 6,
    ThreadStateIdleThreadState = 7,
}