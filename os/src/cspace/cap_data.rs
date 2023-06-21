use common::utils::sign_extend;

pub struct CapData {
    words: [usize; 1],
}

impl CapData {
    pub fn new(data: usize) -> Self {
        CapData { words: [data] }
    }

    pub fn get_guard(&self) -> usize {
        sign_extend(self.words[0] & 0xffffffffffffffc0, 0x0)
    }

    pub fn get_guard_size(&self) -> usize {
        sign_extend(self.words[0] & 0x3f, 0x0)
    }
}