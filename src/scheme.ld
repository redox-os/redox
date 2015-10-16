ENTRY(_start)

MEMORY {
	highmem : org = 0xC0000000, l = 4096K
}

SECTIONS {
	.text : {
		*(.text*)
	} >highmem

	.rodata : {
		*(.rodata*)
	} >highmem

	.data : {
		*(.data*)
	} >highmem

	/DISCARD/ : {
		*(.comment)
		*(.eh_frame)
		*(.rel.eh_frame)
	}
}
