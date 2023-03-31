mod domain_schedule;

use core::sync::atomic::AtomicUsize;
use lazy_static::*;
use spin::Mutex;
use domain_schedule::DomainScheduler;
lazy_static!{
    pub static ref KS_DOM_SCHEDULE: Mutex<[DomainScheduler; 1]> = Mutex::new([DomainScheduler{domain: 0, length: 60}]);
    pub static ref KS_DOM_SCHEDULE_IDX: AtomicUsize = AtomicUsize::new(0);
}