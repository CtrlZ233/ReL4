use common::{types::{VMAttributes, Cptr, CapRights}};
use common::message::{MessageInfo, InvocationLabel::{PageMap, PageTableMap}};

use crate::{set_cap, call_with_mrs, set_mr};

// seL4_RISCV_Page_Map
pub fn sel4_page_map(service: Cptr, vspace: Cptr, vaddr: usize, rights: CapRights, attr: VMAttributes) -> isize {
    let tag = MessageInfo::new(PageMap, 0, 1, 3);
    let mut mr0: usize = vaddr;
    let mut mr1: usize = rights.word[0];
    let mut mr2: usize = attr as usize;
    let mut mr3: usize = 0;
    set_cap(0, vspace);

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.get_label();
    if result != 0 {
        set_mr(0, mr0);
        set_mr(1, mr1);
        set_mr(2, mr2);
        set_mr(3, mr3);
        return -1;
    }

    result as isize
}

// seL4_RISCV_PageTable_Map
pub fn sel4_page_table_map(service: Cptr, vspace: Cptr, vaddr: usize, attr: VMAttributes) -> isize {
    let tag = MessageInfo::new(PageTableMap, 0, 1, 2);
    let mut mr0: usize = vaddr;
    let mut mr1: usize = attr as usize;
    let mut mr2: usize = 0;
    let mut mr3: usize = 0;
    set_cap(0, vspace);

    let output_tag = call_with_mrs(service, tag, &mut mr0, &mut mr1, &mut mr2, &mut mr3);
    let result = output_tag.get_label();
    if result != 0 {
        set_mr(0, mr0);
        set_mr(1, mr1);
        set_mr(2, mr2);
        set_mr(3, mr3);
        return -1;
    }

    result as isize
}