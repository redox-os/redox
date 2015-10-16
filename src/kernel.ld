ENTRY(kernel)

MEMORY {
	lowmem : org = 0xC000, l = 464K
}

SECTIONS {
	.text : {
		*(.text*)
	} >lowmem

	.rodata : {
                *(.rodata*)
        } >lowmem

        .data : {
                *(.data*)
        } >lowmem

	/DISCARD/ : {
		*(.comment)
		*(.eh_frame)
		*(.rel.eh_frame)
	}
}
