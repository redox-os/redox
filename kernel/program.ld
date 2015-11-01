ENTRY(_start)

MEMORY {
	highmem : org = 0x80000000, l = 1024M
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
