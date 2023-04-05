.section .text

.global trap_entry

trap_entry:
    csrrw t0, sscratch, t0

    sd ra, (0*8)(t0)

    sd sp, (1*8)(t0)

    sd gp, (2*8)(t0)
    sd tp, (3*8)(t0)
    sd t1, (5*8)(t0)
    sd t2, (6*8)(t0)
    sd s0, (7*8)(t0)
    sd s1, (8*8)(t0)
    sd a0, (9*8)(t0)
    sd a1, (10*8)(t0)
    sd a2, (11*8)(t0)
    sd a3, (12*8)(t0)
    sd a4, (13*8)(t0)
    sd a5, (14*8)(t0)
    sd a6, (15*8)(t0)
    sd a7, (16*8)(t0)
    sd s2, (17*8)(t0)
    sd s3, (18*8)(t0)
    sd s4, (19*8)(t0)
    sd s5, (20*8)(t0)
    sd s6, (21*8)(t0)
    sd s7, (22*8)(t0)
    sd s8, (23*8)(t0)
    sd s9, (24*8)(t0)
    sd s10, (25*8)(t0)
    sd s11, (26*8)(t0)
    sd t3, (27*8)(t0)
    sd t4, (28*8)(t0)
    sd t5, (29*8)(t0)
    sd t6, (30*8)(t0)

    csrr  x1, sscratch
    sd    x1, (4*8)(t0)
    csrr x1, sstatus
    sd x1, (32*8)(t0)
    csrr s0, scause
    sd s0, (31*8)(t0)

    la sp, boot_stack_top

    csrr x1,  sepc
    sd   x1, (33*8)(t0)
    bltz s0, interrupt
    li   s4, 8
    bne  s0, s4, exception

handle_syscall:
    addi x1, x1, 4
    sd   x1, (34*8)(t0)


    mv a2, a7
    j rust_handle_syscall

exception:
    /* Save NextIP */
    sd   x1, (34*8)(t0)
    j rust_handle_exception
  
interrupt:
    /* Save NextIP */
    sd   x1, (34*8)(t0)
    j rust_handle_interrupt
