ENTRY(_start)
OUTPUT_FORMAT(elf32-i386)

MEMORY {
	highmem : org = 0x80000000, l = 4096K
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
		*(.eh_frame)
	} >highmem

	/DISCARD/ : {
		*(.comment)
		*(.rel.eh_frame)
	}
}
