ENTRY(kernel)

SECTIONS {
    kernel_base = 0x101000;
	. = kernel_base;

    .text : AT(ADDR(.text) - kernel_base) {
        *(.text*)
		. = ALIGN(4096);
    }

	.rodata : AT(ADDR(.rodata) - kernel_base) {
        *(.rodata*)
		. = ALIGN(4096);
    }

    .data : AT(ADDR(.data) - kernel_base) {
        *(.data*)
		. = ALIGN(4096);
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
