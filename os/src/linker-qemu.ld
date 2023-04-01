OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0xffffffff80200000;
OFFSET = 0xffffffff00000000;
SECTIONS
{
    . = BASE_ADDRESS;

    .text : AT(ADDR(.text) - OFFSET) {
        stext = .;
        *(.text.entry)
        *(.text .text.*)
        etext = .;
    }

    . = ALIGN(4K);
    .rodata : AT(ADDR(.rodata) - OFFSET) {
        srodata = .;
        *(.rodata .rodata.*)
        erodata = .;
    }

    . = ALIGN(4K);
    .data : AT(ADDR(.data) - OFFSET) {
        sdata = .;
        *(.data .data.*)
        edata = .;
    }

    . = ALIGN(4K);
    .kernel : AT(ADDR(.kernel) - OFFSET) {
        *(.kernel.idle_thread)
    }

    . = ALIGN(4K);
    sbss_with_stack = .;
    .bss : AT(ADDR(.bss) - OFFSET) {
        *(.bss.root_pagetable)
        *(.bss.l2_pagetable)
	    *(.bss.stack)
        sbss = .;
        *(.sbss .bss .bss.*)
        ebss = .;
    }
    . = ALIGN(4K);

    kernel_end = .;
    PROVIDE(end = .);
}
