use crate::config::CONFIG_ROOT_CNODE_SIZE_BITS;
use crate::cspace::cap::{Cap, CapTableEntry, MDBNode};
use crate::utils::{bit};

pub struct CNode {
    cnode: [CapTableEntry; 1 << CONFIG_ROOT_CNODE_SIZE_BITS],
}

pub enum CNodeSlot {
    SeL4CapNull =  0,                   /* null cap */
    SeL4CapInitThreadTcb =  1,          /* initial thread's TCB cap */
    SeL4CapInitThreadCNode =  2,        /* initial thread's root CNode cap */
    SeL4CapInitThreadVspace =  3,       /* initial thread's VSpace cap */
    SeL4CapIrqControl =  4,             /* global IRQ controller cap */
    SeL4CapASIDControl =  5,            /* global ASID controller cap */
    SeL4CapInitThreadASIDPool =  6,     /* initial thread's ASID pool cap */
    SeL4CapIOPortControl =  7,          /* global IO port control cap (null cap if not supported) */
    SeL4CapIOSpace =  8,                /* global IO space cap (null cap if no IOMMU support) */
    SeL4CapBootInfoFrame =  9,          /* bootinfo frame cap */
    SeL4CapInitThreadIpcBuffer = 10,    /* initial thread's IPC buffer frame cap */
    SeL4CapDomain = 11,                 /* global domain controller cap */
    SeL4CapSMMUSIDControl = 12,         /*global SMMU SID controller cap, null cap if not supported*/
    SeL4CapSMMUCBControl = 13,          /*global SMMU CB controller cap, null cap if not supported*/
    SeL4NumInitialCaps = 14
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