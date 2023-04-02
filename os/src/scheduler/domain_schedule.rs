use core::sync::atomic::AtomicUsize;
use crate::config::CONFIG_NUM_PRIORITIES;
use crate::types::Dom;

pub static KS_CUR_DOMAIN: AtomicUsize = AtomicUsize::new(0);
pub static KS_DOMAIN_TIME: AtomicUsize = AtomicUsize::new(0);
pub struct DomainScheduler {
    pub domain: Dom,
    pub length: usize,
}

pub enum PriorityConst {
    InvalidPrio = -1,
    MinPrio = 0,
    MaxPrio = CONFIG_NUM_PRIORITIES as isize- 1,
}