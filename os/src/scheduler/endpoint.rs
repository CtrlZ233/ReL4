use crate::scheduler::tcb::TCBQueue;
use common::types::Pptr;
use common::utils::sign_extend;

pub struct EndPoint {
    words: [usize; 2],
}

impl EndPoint {
    pub fn get_queue_head(&self) -> Pptr {
        sign_extend(self.words[1] & 0xffffffffffffffff, 0x0)
    }

    pub fn set_queue_head(&mut self, pptr: Pptr) {
        self.words[1] &= !0xffffffffffffffff;
        self.words[1] |= pptr & 0xffffffffffffffff;
    }

    pub fn get_queue(&self) -> TCBQueue {
        TCBQueue::new(self.get_queue_head(), self.get_queue_tail())
    }

    pub fn set_queue(&mut self, queue: &TCBQueue) {
        self.set_queue_head(queue.head as Pptr);
        self.set_queue_tail(queue.end as Pptr);
    }

    pub fn get_queue_tail(&self) -> Pptr {
        sign_extend(self.words[0] & 0x7ffffffffc, 0xffffff8000000000)
    }

    pub fn set_queue_tail(&mut self, pptr: Pptr) {
        self.words[0] &= !0x7ffffffffc;
        self.words[0] |= pptr & 0x7ffffffffc;
    }

    pub fn get_state(&self) -> EndPointState {
        unsafe {
            core::mem::transmute::<u8, EndPointState>(sign_extend(self.words[0] & 0x3, 0x0) as u8)
        }
    }

    pub fn set_state(&mut self, state: EndPointState) {
        self.words[0] &= !0x3;
        self.words[0] |= (state as usize) & 0x3;
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum EndPointState {
    EPStateIdle = 0,
    EPStateSend = 1,
    EPStateRecv = 2,
}