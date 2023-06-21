use common::utils::{bool2usize, sign_extend};


#[derive(Copy, Clone)]
pub struct MDBNode {
    words:[usize; 2]
}

impl MDBNode {
    pub fn new(mdb_next: usize, mdb_revocable: bool, mdb_first_badged: bool, mdb_prev: usize) -> Self {
        let mut mdb_node = MDBNode {words: [0, 0]};
        mdb_node.words[0] = 0
            | mdb_prev << 0;
        mdb_node.words[1] = 0
            | (mdb_next & 0x7ffffffffc) >> 0
            | (bool2usize(mdb_revocable) & 0x1) << 1
            | (bool2usize(mdb_first_badged) & 0x1) << 0;
        mdb_node
    }
    pub fn null_mdbnode() -> Self {
        Self::new(0, false, false, 0)
    }

    pub fn set_mdb_prev(&mut self, v64: usize) {
        self.words[0] &= !0xffffffffffffffff;
        self.words[0] |= v64;
    }

    pub fn set_mdb_revocable(&mut self, mdb_revocable: bool) {
        self.words[1] &= !(0x2 as usize);
        self.words[1] |= (bool2usize(mdb_revocable) << 1) & (0x2 as usize);
    }

    pub fn set_mdb_first_badged(&mut self, mdb_first_badged: bool) {
        self.words[1] &= !(0x1 as usize);
        self.words[1] |= (bool2usize(mdb_first_badged) << 0) & (0x1 as usize);
    }

    pub fn set_mdb_next(&mut self, v64: usize) {
        self.words[1] &= !0x7ffffffffc;
        self.words[1] |= v64 & 0x7ffffffffc;
    }

    pub fn get_mdb_prev(&self) -> usize {
        sign_extend(self.words[0] & 0xffffffffffffffff, 0x0)
    }

    pub fn get_mdb_next(&self) -> usize {
        sign_extend(self.words[1] & 0x7ffffffffc, 0xffffff8000000000)
    }

    pub fn get_mdb_revocable(&self) -> bool {
        sign_extend((self.words[1] & 0x2) >> 1, 0x0) == 1
    }

    pub fn get_mdb_first_badged(&self) -> bool {
        sign_extend(self.words[1] & 0x1, 0x0) == 1
    }
}