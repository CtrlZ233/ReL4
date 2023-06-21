use core::ops::{Index, IndexMut};
use common::config::CONFIG_ROOT_CNODE_SIZE_BITS;
use crate::cspace::{cap::{Cap, CapTableEntry}, mdb::MDBNode};
use common::utils::bit;

pub struct CNode {
    cnode: [CapTableEntry; 1 << CONFIG_ROOT_CNODE_SIZE_BITS],
}

impl CNode {
    pub fn write(&mut self, index: usize, cap: Cap) {
        assert!(index < bit(CONFIG_ROOT_CNODE_SIZE_BITS));
        self.cnode[index].cap = cap;
        self.cnode[index].mdb_node = MDBNode::null_mdbnode();
        self.cnode[index].mdb_node.set_mdb_revocable(true);
        self.cnode[index].mdb_node.set_mdb_first_badged(true);
    }
}

impl Index<usize> for CNode {
    type Output = CapTableEntry;

    fn index(&self, index: usize) -> &Self::Output {
        &self.cnode[index]
    }
}

impl IndexMut<usize> for CNode {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.cnode[index]
    }
}

pub enum TCBCNodeIndex {
    TCBCTable = 0,
    TCBVTable = 1,
    TCBReply = 2,
    TCBCaller = 3,
    TCBBuffer = 4,
    TCBCNodeEntries = 5,
}