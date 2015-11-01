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

        .eh_frame : {
		*(.eh_frame*)
        } >highmem

	/DISCARD/ : {
		*(.comment)
		*(.rel.eh_frame)
	}
}
