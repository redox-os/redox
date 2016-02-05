ENTRY(kernel)

SECTIONS {
	. = 0xC000;

    .text : AT(ADDR(.text) - 0xC000) {
        *(.text*)
		. = ALIGN(4096);
    }

	.rodata : AT(ADDR(.rodata) - 0xC000) {
        *(.rodata*)
		. = ALIGN(4096);
    }

    .data : AT(ADDR(.data) - 0xC000) {
        *(.data*)
		. = ALIGN(4096);
    }

    .bss : AT(ADDR(.bss) - 0xC000) {
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
