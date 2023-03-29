use core::cmp::min;

use log::{error, debug};
use spin::MutexGuard;

use crate::{mm::{Region, PhyRegion, VirtRegion}, config::{KERNEL_ELF_BASE, PV_BASE_OFFSET, MAX_NUM_FREEMEM_REG, AVAIL_MEM_DEVICE, MAX_NUM_RESV_REG, PPTR_BASE_OFFSET}};
use super::{RES_REG, NDKS_BOOT, AVAIL_REG, AVAIL_P_REGS, ndks_boot::NdksBoot};


pub fn init(ui_reg: Region, it_v_reg: VirtRegion) {
    let mut index = 1;
    unsafe {
        let mut res_reg = RES_REG.lock();
        res_reg[0].start = KERNEL_ELF_BASE - PV_BASE_OFFSET + PPTR_BASE_OFFSET;
        extern "C" {
            fn kernel_end();
        }
        res_reg[0].end = kernel_end as usize - PV_BASE_OFFSET + PPTR_BASE_OFFSET;
        res_reg[index] = ui_reg;
        index += 1;
    }

    
    {
        let res_reg = RES_REG.lock();
        for i in 0..index {
            debug!("reserved_{}: {:#x} ... {:#x}", i, res_reg[i].start, res_reg[i].end);
        }
    }

    init_freemem(AVAIL_MEM_DEVICE, index, it_v_reg);
}

fn init_freemem(n_available: usize, n_reserved: usize, it_v_reg: VirtRegion) {
    let reserved = RES_REG.lock();
    let mut ndks_boot = NDKS_BOOT.lock();
    let mut avail_reg = AVAIL_REG.lock();
    let avail_p_regs = AVAIL_P_REGS.lock();

    for i in 0..AVAIL_MEM_DEVICE {
        avail_reg[i] = Region::paddr_to_pptr_reg(avail_p_regs[i]);
    }

    let mut a = 0;
    let mut r = 0;
    while a < n_available && r < n_reserved {
        if reserved[r].start == reserved[r].end {
            r += 1;
        } else if avail_reg[a].start >= avail_reg[a].end {
            a += 1;
        } else if reserved[r].end <= avail_reg[a].start {
            reserve_region(PhyRegion::pptr_to_paddr_reg(reserved[r]), &mut ndks_boot);
            r += 1;
        } else if reserved[r].start >= avail_reg[a].end {
            insert_region(avail_reg[a], &mut ndks_boot);
            a += 1;
        } else {
            if reserved[r].start <= avail_reg[a].start {
                avail_reg[a].start = min(avail_reg[a].end, reserved[r].end);
                reserve_region(PhyRegion::pptr_to_paddr_reg(reserved[r]), &mut ndks_boot);
                r += 1;
            } else {
                assert!(reserved[r].start < avail_reg[a].end);
                let mut m = avail_reg[a];
                m.end = reserved[r].start;
                insert_region(m, &mut ndks_boot);
                if avail_reg[a].end >= reserved[r].end {
                    avail_reg[a].start = reserved[r].end;
                    reserve_region(PhyRegion::pptr_to_paddr_reg(reserved[r]), &mut ndks_boot);
                    r += 1;
                } else {
                    a += 1;
                }
            }
        }
    }

    while r < n_reserved {
        if reserved[r].start < reserved[r].end {
            reserve_region(PhyRegion::pptr_to_paddr_reg(reserved[r]), &mut ndks_boot);
        }
        r += 1;
    }

    while a < n_available {
        if avail_reg[a].start < avail_reg[a].end {
            insert_region(avail_reg[a], &mut ndks_boot);
        }
        a += 1;
    }

    for i in 0..MAX_NUM_FREEMEM_REG {
        debug!("ndks_boot.freemem_{}: {:#x} ... {:#x}", i, ndks_boot.freemem[i].start, ndks_boot.freemem[i].end);
    }

}

fn reserve_region(reg: PhyRegion, ndks_boot: &mut MutexGuard<NdksBoot>) {
    
    let mut i = 0;
    assert!(reg.start <= reg.end);
    if is_reg_empty(reg) {
        return;
    }

    while i < ndks_boot.resv_count {
        if ndks_boot.reserved[i].start == reg.end {
            ndks_boot.reserved[i].start = reg.start;
            merge_regions(ndks_boot);
            return;
        }

        if ndks_boot.reserved[i].end == reg.start {
            ndks_boot.reserved[i].end = reg.end;
            merge_regions(ndks_boot);
            return;
        }

        if ndks_boot.reserved[i].start > reg.end {
            if ndks_boot.resv_count + 1 > MAX_NUM_RESV_REG {
                error!("[reserve_region]error!");
                assert!(1 == 0);
            }

            let mut j = ndks_boot.resv_count;
            while j > i {
                ndks_boot.reserved[j] = ndks_boot.reserved[j - 1];
                j -= 1;
            }
            ndks_boot.reserved[i] = reg;
            ndks_boot.resv_count += 1;
            return;
        }

        i += 1;
    }
    if i + 1 == MAX_NUM_RESV_REG {
        error!("[reserve_region]error!");
        assert!(1 == 0);
    }

    ndks_boot.reserved[i] = reg;
    ndks_boot.resv_count += 1;
}

fn merge_regions(ndks_boot: &mut MutexGuard<NdksBoot>) {
    let mut i = 1;
    while i < ndks_boot.resv_count {
        if ndks_boot.reserved[i - 1].end == ndks_boot.reserved[i].start {
            ndks_boot.reserved[i - 1].end = ndks_boot.reserved[i].end;
            let mut j = i + 1;
            while j < ndks_boot.resv_count {
                ndks_boot.reserved[j - 1] = ndks_boot.reserved[j];
                j += 1;
            }
            ndks_boot.resv_count -= 1;
        } else {
            i += 1;
        }
    }
}

fn insert_region(reg: Region, ndks_boot: &mut MutexGuard<NdksBoot>) {
    assert!(reg.start <= reg.end);
    if is_reg_empty(reg) {
        return;
    }

    for i in 0..MAX_NUM_FREEMEM_REG {
        if is_reg_empty(ndks_boot.freemem[i]) {
            reserve_region(PhyRegion::pptr_to_paddr_reg(reg), ndks_boot);
            ndks_boot.freemem[i] = reg;
            return;
        }
    }
    error!("[insert_region] error!");
    assert!(1 == 0);
}

#[inline]
fn is_reg_empty(reg: Region) -> bool {
    reg.start == reg.end
}
