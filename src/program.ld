ENTRY(entry)
OUTPUT_FORMAT(elf32-i386)

MEMORY {
	highmem : org = 0x80000000, l = 4096K
}

SECTIONS {
	.text : {
		*(.text*)
	} >highmem

	.data : {
		*(.data*)
	} >highmem

	.rodata : {
		*(.rodata*)
	} >highmem

	/DISCARD/ : {
		*(.comment)
		*(.eh_frame)
		*(.rel.eh_frame)
	}
}
