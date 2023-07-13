mod timer;

use common::register::Register;
use log::error;
use riscv::register::{scause::{self, Interrupt, Trap, Exception}, stval};

use crate::{scheduler::{timer_tick, schedule, activate_thread, get_current_tcb}, trap::restore_user_context, sbi::shutdown};

use self::timer::set_next_trigger;

pub fn init() {
    timer::init();
}

pub fn handle_interrupt() {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            timer_tick();
            set_next_trigger();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.",
                scause.cause(),
                stval,
                get_current_tcb().get_register(Register::FaultIP as usize),
            );
            shutdown(true)
        }
        _ => {
            panic!("invaild interrrupt");
        }
    }
    schedule();
    activate_thread();
    restore_user_context();
}