    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top

set_boot_pt:
    la  t0, boot_page_table_sv39
    srli t0, t0, 12
    li   t1, 8 << 60
    or   t0, t0, t1
    csrw satp, t0
    sfence.vma
    la   t0, rust_main
    li   t1, 0xffffffff00000000
    add  t0, t0, t1
    add  sp, sp, t1
    jr t0
    # call   rust_main

    .section .bss.boot.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:

    .section .data
    .align 12
boot_page_table_sv39:
    .quad 0
    .quad 0
    # 0x00000000_80000000 -> 0x80000000 (1G, VRWXAD)
    .quad (0x80000 << 10) | 0xcf
    # removed
    #.quad 0
    .zero 8 * 507
    # 0xffffffff_80000000 -> 0xffffffff_80000000 (1G, VRWXAD)
    .quad (0x80000 << 10) | 0xcf
    .quad 0