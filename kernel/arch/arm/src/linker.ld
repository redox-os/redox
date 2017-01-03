ENTRY(kstart)
OUTPUT_ARCH(arm)
OUTPUT_FORMAT(elf32-littlearm)

KERNEL_OFFSET = 0;

SECTIONS {
    . = KERNEL_OFFSET;

    .text : AT(ADDR(.text) - KERNEL_OFFSET) {
        __text_start = .;
        *(.text*)
	. = ALIGN(4096);
        __text_end = .;
    }

	.rodata : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        __rodata_start = .;
        *(.rodata*)
	. = ALIGN(4096);
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - KERNEL_OFFSET) {
        __data_start = .;
        *(.data*)
	. = ALIGN(4096);
        __data_end = .;
    }

    .tdata : AT(ADDR(.tdata) - KERNEL_OFFSET) {
        __tdata_start = .;
        *(.tdata*)
        . = ALIGN(4096);
        __tdata_end = .;
        __tbss_start = .;
        *(.tbss*)
        . += 8;
        . = ALIGN(4096);
        __tbss_end = .;
    }

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET) {
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    }

    __end = .;

    /DISCARD/ : {
        *(.comment*)
        *(.debug*)
        *(.eh_frame*)
        *(.gcc_except_table*)
        *(.note*)
        *(.rel.eh_frame*)
    }
}
