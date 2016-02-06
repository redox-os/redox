ENTRY(kernel)

SECTIONS {
	. = 0x100000;

    .text : AT(ADDR(.text) - 0x100000) {
        *(.text*)
		. = ALIGN(4096);
    }

	.rodata : AT(ADDR(.rodata) - 0x100000) {
        *(.rodata*)
		. = ALIGN(4096);
    }

    .data : AT(ADDR(.data) - 0x100000) {
        *(.data*)
		. = ALIGN(4096);
    }

    .bss : AT(ADDR(.bss) - 0x100000) {
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
