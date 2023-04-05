mod domain_schedule;
mod tcb;
mod register;
mod scheduler;

use core::sync::atomic::AtomicUsize;
use lazy_static::*;
use log::{debug, error};
use core::arch::asm;
use core::ops::IndexMut;
use core::sync::atomic::Ordering::SeqCst;
use spin::Mutex;
use domain_schedule::DomainScheduler;

pub use tcb::{TCB, IdleTCB};

use crate::{config::{CPU_NUM, SEL4_IDLE_TCB_SLOT_SIZE, TCB_OFFSET, CONFIG_KERNEL_STACK_BITS}, types::Pptr};
use crate::config::PPTR_BASE_OFFSET;
use crate::cspace::{Cap, CNode, create_init_thread_cap, cte_insert, derive_cap, TCBCNodeIndex};
use crate::cspace::CapTag::CapPageTableCap;
use crate::cspace::CNodeSlot::{SeL4CapInitThreadCNode, SeL4CapInitThreadIpcBuffer, SeL4CapInitThreadVspace};
use crate::cspace::TCBCNodeIndex::TCBVTable;
use crate::mm::set_vspace_root;
use crate::root_server::ROOT_SERVER;
use crate::scheduler::domain_schedule::{KS_CUR_DOMAIN, KS_DOMAIN_TIME, PriorityConst};
use crate::scheduler::register::CAP_REGISTER;
use crate::scheduler::tcb::TCBCNode;
use crate::scheduler::tcb::ThreadStateEnum::ThreadStateRunning;
use crate::types::Vptr;
use crate::utils::hart_id;
lazy_static!{
    pub static ref KS_DOM_SCHEDULE: Mutex<[DomainScheduler; 1]> = Mutex::new([DomainScheduler{domain: 0, length: 60}]);
    pub static ref KS_DOM_SCHEDULE_IDX: AtomicUsize = AtomicUsize::new(0);
}

#[no_mangle]
#[link_section = ".kernel.idle_thread"]
static mut KS_IDLE_THREAD_TCB: [IdleTCB; CPU_NUM] = [IdleTCB {array: [0; SEL4_IDLE_TCB_SLOT_SIZE]}; CPU_NUM];

static mut KS_IDLE_THREAD: [Pptr; CPU_NUM] = [0; CPU_NUM];
static mut KERNEL_STACK: [[u8; 1 << CONFIG_KERNEL_STACK_BITS]; CPU_NUM] = [[0; 1 << CONFIG_KERNEL_STACK_BITS]; CPU_NUM];
pub static mut KS_CUR_THREAD: [Pptr; CPU_NUM] = [0; CPU_NUM];
static mut KS_SCHEDULER_ACTION: [Pptr; CPU_NUM] = [0; CPU_NUM];
const SCHEDULER_ACTION_RESUME_CURRENT_THREAD: usize = 0;
const SCHEDULER_ACTION_CHOOSE_NEW_THREAD: usize = 1;

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

pub fn create_initial_thread(root_cnode_cap: Cap, vspace_cap: Cap, v_entry: Vptr, bi_frame_vptr: Vptr,
                             ipc_buf_vptr: Vptr, ipc_buf_cap: Cap) -> *const TCB {
    debug!("root_server_tcb pptr: {:#x}", ROOT_SERVER.lock().tcb);
    let tcb = unsafe {
        &mut *((ROOT_SERVER.lock().tcb + TCB_OFFSET) as *mut TCB)
    };
    tcb.init_context();
    let root_cnode = unsafe {
        &mut *(root_cnode_cap.get_cap_pptr() as *mut CNode)
    };
    let (derive_ret, new_cap) = derive_cap(&mut root_cnode[SeL4CapInitThreadIpcBuffer as usize], ipc_buf_cap);
    if !derive_ret {
        error!("Failed to derive copy of IPC Buffer");
        assert_eq!(1, 0);
    }

    let tcb_cnode = unsafe {
        &mut *(ROOT_SERVER.lock().tcb as *mut TCBCNode)
    };

    cte_insert(root_cnode_cap, &mut root_cnode[SeL4CapInitThreadCNode as usize],
               &mut tcb_cnode[TCBCNodeIndex::TCBCTable as usize]);
    cte_insert(vspace_cap, &mut root_cnode[SeL4CapInitThreadVspace as usize],
               &mut tcb_cnode[TCBVTable as usize]);
    debug!("tcb_cnode_addr: {:#x}", tcb_cnode as *mut TCBCNode as usize);
    assert_eq!(tcb_cnode[TCBVTable as usize].cap.get_cap_type(), CapPageTableCap);
    cte_insert(new_cap, &mut root_cnode[SeL4CapInitThreadIpcBuffer as usize],
               &mut tcb_cnode[TCBCNodeIndex::TCBBuffer as usize]);
    tcb.tcb_ipc_buffer = ipc_buf_vptr;
    tcb.set_register(CAP_REGISTER, bi_frame_vptr);
    tcb.set_next_pc(v_entry);
    tcb.tcb_priority = PriorityConst::MaxPrio as usize;
    tcb.tcb_mcp = PriorityConst::MaxPrio as usize;
    tcb.tcb_domain = KS_DOM_SCHEDULE.lock()[KS_DOM_SCHEDULE_IDX.load(SeqCst)].domain;

    tcb.setup_replay_master();
    tcb.set_thread_state(ThreadStateRunning);

    KS_CUR_DOMAIN.store(KS_DOM_SCHEDULE.lock()[KS_DOM_SCHEDULE_IDX.load(SeqCst)].domain, SeqCst);
    KS_DOMAIN_TIME.store(KS_DOM_SCHEDULE.lock()[KS_DOM_SCHEDULE_IDX.load(SeqCst)].length, SeqCst);

    let it_cap = create_init_thread_cap(root_cnode_cap, tcb as *const TCB as Pptr);

    tcb as *const TCB
}

pub fn init_core_state(tcb: *const TCB) {
    unsafe {
        KS_SCHEDULER_ACTION[hart_id()] = tcb as Pptr;
        debug!("KS_SCHEDULER_ACTION[hart_id()]: {:#x}", KS_SCHEDULER_ACTION[hart_id()]);
        KS_CUR_THREAD[hart_id()] = KS_IDLE_THREAD[hart_id()];
    }
}

pub fn choose_thread() {
    let dom = 0;
    
}

pub fn schedule_choose_new_thread() {
    if KS_DOMAIN_TIME.load(SeqCst) == 0 {
        // TODO next_domain()
    }
    choose_thread();
}

pub fn schedule() {
    unsafe {
        if KS_SCHEDULER_ACTION[hart_id()] != SCHEDULER_ACTION_RESUME_CURRENT_THREAD {
            let mut was_runable = false;
            let ks_cur_thread = &mut *(KS_CUR_THREAD[hart_id()] as *mut TCB);
            if ks_cur_thread.is_schedulable() {
                was_runable = true;
                //TODO: enqueue current tcb
            }

            if KS_SCHEDULER_ACTION[hart_id()] == SCHEDULER_ACTION_CHOOSE_NEW_THREAD {
                schedule_choose_new_thread()
            } else {
                let tcb = &mut *(KS_SCHEDULER_ACTION[hart_id()] as *mut TCB);
                debug!("KS_SCHEDULER_ACTION[hart_id()]: {:#x}", KS_SCHEDULER_ACTION[hart_id()]);
                assert!(tcb.is_schedulable());
                //TODO: judge fast path

                assert_ne!(KS_SCHEDULER_ACTION[hart_id()], KS_CUR_THREAD[hart_id()]);
                switch_to_thread(tcb);
            }
        }
    }
}

pub fn set_vm_root(tcb: & TCB) {
    let tcb_cnode = unsafe {
        &mut *(tcb.get_cnode_ptr_of_this() as *mut TCBCNode)
    };

    let thread_root = tcb_cnode[TCBVTable as usize].cap;
    let lvl1pt =thread_root.get_pt_based_ptr();
    let asid = thread_root.get_pt_mapped_asid();
    // TODO: check page table & asid

    set_vspace_root(lvl1pt - PPTR_BASE_OFFSET, asid);
}

pub fn switch_to_thread(tcb: &mut TCB) {
    set_vm_root(tcb);
    // TODO: dequeue current tcb
    unsafe {
        KS_CUR_THREAD[hart_id()] = tcb as *const TCB as usize;
    }
}