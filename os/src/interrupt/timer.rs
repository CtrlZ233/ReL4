use common::config::{CLOCK_FREQ, TICKS_PER_SEC};
use riscv::register::{sie, time};

use crate::sbi::set_timer;

pub fn init() {
    unsafe {
        sie::set_stimer();
    }
    set_next_trigger();
}

pub fn get_time() -> usize {
    time::read()
}

pub fn set_next_trigger() {
    set_timer(get_time() + CLOCK_FREQ / TICKS_PER_SEC)
}