ENTRY(kernel)

SECTIONS {
    kernel_base = 0x101000;
	. = kernel_base;

    .text : AT(ADDR(.text) - kernel_base) {
        __text_start = .;
        *(.text*)
		. = ALIGN(4096);
        __text_end = .;
    }

	.rodata : AT(ADDR(.rodata) - kernel_base) {
        __rodata_start = .;
        *(.rodata*)
		. = ALIGN(4096);
        __rodata_end = .;
    }

    .data : AT(ADDR(.data) - kernel_base) {
        __data_start = .;
        *(.data*)
		. = ALIGN(4096);
        __data_end = .;
    }

    .bss : AT(ADDR(.bss) - kernel_base) {
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    }

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
        *(.rel.eh_frame)
    }
}
