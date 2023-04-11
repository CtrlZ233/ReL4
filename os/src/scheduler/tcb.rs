use crate::{config::{CONTEXT_REGISTERS_NUM, SEL4_IDLE_TCB_SLOT_SIZE, CONFIG_KERNEL_STACK_BITS}, types::{Pptr, Dom, Prio, Cptr, Vptr}, utils::{bit, hart_id, sign_extend}};
use core::ops::{Index, IndexMut};
use super::{register::{Register, SSTATUS_SPP, SSTATUS_SPIE, SP}, idle_thread, KERNEL_STACK, KS_CUR_THREAD, KS_SCHEDULER_ACTION, SCHEDULER_ACTION_RESUME_CURRENT_THREAD};

use log::{error, debug};
use crate::config::{SEL4_TCB_BITS, VM_READ_ONLY, VM_READ_WRITE, WORD_BITS};
use crate::cspace::{Cap, CapTableEntry, CapTag, MDBNode, resolve_address_bits};
use crate::cspace::TCBCNodeIndex::{TCBBuffer, TCBCTable, TCBReply};
use crate::scheduler::endpoint::{EndPoint, EndPointState};
use crate::scheduler::Register::FaultIP;
use crate::scheduler::register::Register::{NextIP, SSTATUS};
use crate::scheduler::ThreadStateEnum::{ThreadStateInactive, ThreadStateRunning};
use crate::utils::{mask, page_bits_for_size};

#[derive(Default)]
pub struct TCB {
    context: RiscvContext,
    tcb_state: ThreadState,
    pub tcb_bound_notification: Pptr,
    tcb_fault: Fault,
    lookup_fault: LookUpFault,
    pub tcb_domain: Dom,
    pub tcb_mcp: Prio,
    pub tcb_priority: Prio,
    pub tcb_time_slice: usize,
    pub tcb_fault_handler: Cptr,
    pub tcb_ipc_buffer: Vptr,

    pub tcb_sched_next: Pptr,
    pub tcb_sched_prev: Pptr,

    pub tcb_ep_next: Pptr,
    pub tcb_ep_prev: Pptr,
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

    pub fn suspend(&mut self) {
        self.cancel_ipc();
        if self.get_state() == ThreadStateRunning {
            self.update_restart_pc();
        }
        self.set_thread_state(ThreadStateInactive);
    }

    pub fn update_restart_pc(&mut self) {
        self.set_register(FaultIP as usize, self.get_register(NextIP as usize));
    }

    pub fn cancel_ipc(&mut self) {
        match self.get_state() {
            ThreadStateEnum::ThreadStateBlockedOnSend | ThreadStateEnum::ThreadStateBlockedOnReceive => {
                let epptr = self.tcb_state.get_blocking_object();
                let endpoint_ref = unsafe {
                    &mut *(epptr as *mut EndPoint)
                };
                assert_ne!(endpoint_ref.get_state(), EndPointState::EPStateIdle);

                let mut queue = endpoint_ref.get_queue();
                queue.de_queue(self);
                endpoint_ref.set_queue(&queue);
                if queue.head as Pptr != 0 {
                    endpoint_ref.set_state(EndPointState::EPStateIdle);
                }

                self.set_thread_state(ThreadStateInactive);

            }
            _ => {
                // TODO: more state cancel
            }
        }
    }

    pub fn de_queue_from_sched(&mut self) {
        if self.tcb_state.is_get_tcb_queued() {
            let dom = self.tcb_domain;
            let prio = self.tcb_priority;

        }
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

    pub fn get_cnode_ptr_of_this(&self) -> Pptr {
        let self_ptr = self as *const TCB as Pptr;
        self_ptr & !(mask(SEL4_TCB_BITS))
    }

    pub fn get_register(&self, index: usize) -> usize {
        self.context.registers[index]
    }

    pub fn get_context_base_ptr(&self) -> Pptr {
        &(self.context) as *const RiscvContext as usize
    }

    pub fn set_register(&mut self, reg: usize, w: usize) {
        self.context.registers[reg] = w;
    }

    pub fn set_next_pc(&mut self, entry: usize) {
        self.context.registers[NextIP as usize] = entry;
    }

    pub fn setup_replay_master(&mut self) {
        let cnode = unsafe {
            &mut *(self.get_cnode_ptr_of_this() as *mut TCBCNode)
        };
        let slot = &mut cnode[TCBReply as usize];
        if slot.cap.get_cap_type() == CapTag::CapNullCap {
            slot.cap = Cap::new_reply_cap(true, true, self as *const TCB as Pptr);
            slot.mdb_node = MDBNode::null_mdbnode();
            slot.mdb_node.set_mdb_revocable(true);
            slot.mdb_node.set_mdb_first_badged(true);
        }
    }

    pub fn init_context(&mut self) {
        self.context.registers[SSTATUS as usize] = SSTATUS_SPIE;
    }

    pub fn lookup_slot(&self, cap_ptr: usize) -> Option<*mut CapTableEntry> {
        let thread_root_cap = unsafe {
            (&mut *(self.get_cnode_ptr_of_this() as *mut TCBCNode))[TCBCTable as usize].cap
        };
        resolve_address_bits(thread_root_cap, cap_ptr, bit(WORD_BITS))
    }

    pub fn lookup_cap_and_slot(&self, cap_ptr: usize) -> Option<(Cap, *mut CapTableEntry)> {
        match self.lookup_slot(cap_ptr) {
            Some(slot) => {
                unsafe {
                    Some(((&mut *slot).cap, slot))
                }
            }
            _ => {
                None
            }
        }
    }

    pub fn lookup_ipc_buffer(&self, is_receiver: bool) -> Option<Pptr> {
        let w_buffer_ptr = self.tcb_ipc_buffer;
        let buffer_cap = unsafe {
            (&mut *(self.get_cnode_ptr_of_this() as *mut TCBCNode))[TCBBuffer as usize].cap
        };

        if buffer_cap.get_cap_type() != CapTag::CapFrameCap {
            return None;
        }
        if buffer_cap.get_frame_is_device() {
            error!("is device frame");
            return None;
        }
        let vm_right = buffer_cap.get_frame_frame_vm_right();

        if vm_right == VM_READ_WRITE || (!is_receiver && vm_right == VM_READ_ONLY) {
            let base_ptr = buffer_cap.get_frame_base_ptr();
            let page_bits = page_bits_for_size(buffer_cap.get_frame_size());
            return Some(base_ptr + (w_buffer_ptr & mask(page_bits)));
        }

        None

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
        self.words[0] |= (ts << 0) &0xf;

    }

    pub fn get_blocking_object(&self) -> Pptr {
        sign_extend(self.words[0] & 0x7ffffffff0, 0xffffff8000000000)
    }

    pub fn is_get_tcb_queued(&self) -> bool {
        sign_extend(self.words[1] & 0x1, 0x0) == 1
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

pub type TCBCNode = [CapTableEntry; 16];

#[derive(PartialEq, Eq)]
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

pub struct TCBQueue {
    pub head: *mut TCB,
    pub end: *mut TCB,
}

impl TCBQueue {
    pub fn new(head: Pptr, end: Pptr) -> Self {
        TCBQueue {
            head: head as *mut TCB,
            end: end as *mut TCB,
        }
    }

    pub fn de_queue(&mut self, tcb: &mut TCB) {
        if tcb.tcb_ep_prev != 0 {
            let prev = unsafe {
                &mut *(tcb.tcb_ep_prev as *mut TCB)
            };
            prev.tcb_ep_next = tcb.tcb_ep_next;
        } else {
            self.head = tcb.tcb_ep_next as *mut TCB;
        }

        if tcb.tcb_ep_next != 0 {
            let next = unsafe {
                &mut *(tcb.tcb_ep_next as *mut TCB)
            };
            next.tcb_ep_prev = tcb.tcb_ep_prev;
        } else {
            self.end = tcb.tcb_ep_prev as *mut TCB;
        }
    }
}
