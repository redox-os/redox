ENTRY(kstart)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_OFFSET = 0x100000;

SECTIONS {
    . = KERNEL_OFFSET;

    . += SIZEOF_HEADERS;
    . = ALIGN(4096);

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
    }

    .tbss : AT(ADDR(.tbss) - KERNEL_OFFSET) {
        __tbss_start = .;
        *(.tbss*)
        . = ALIGN(4096);
        __tbss_end = .;
    }

    .tcb : AT(ADDR(.tcb) - KERNEL_OFFSET) {
        __tcb = .;
        . += 8;
        . = ALIGN(4096);
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
