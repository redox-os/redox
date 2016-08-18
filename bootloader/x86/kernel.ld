ENTRY(kstart)
OUTPUT_FORMAT(elf64-x86-64)

KERNEL_OFFSET = 0;

SECTIONS {
    . = 0x100000;

    . += SIZEOF_HEADERS;
    . = ALIGN(4096);

	. += KERNEL_OFFSET;

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

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET) {
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.comment*)
        *(.eh_frame*)
        *(.note*)
        *(.rel.eh_frame*)
    }
}
