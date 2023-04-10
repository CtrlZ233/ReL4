use super::tcb::TCB;
use common::config::{CONFIG_NUM_PRIORITIES, CONFIG_NUM_DOMAINS};
struct TCBQueue {
    head: *mut TCB,
    end: *mut TCB,
}

struct Scheduler {
    ready_queues: [TCBQueue; CONFIG_NUM_PRIORITIES * CONFIG_NUM_DOMAINS],
    ready_queues_l1_bitmap: [usize; CONFIG_NUM_DOMAINS],
    ready_queues_l2_bitmap: [usize; CONFIG_NUM_DOMAINS],

}