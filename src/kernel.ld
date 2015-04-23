ENTRY(kernel)
OUTPUT_FORMAT(elf32-i386)

MEMORY {
	lowmem : org = 0xC000, l = 256K
}

SECTIONS {
	.text : {
		*(.text*)
	} >lowmem

	.data : {
		*(.data*)
	} >lowmem

	.rodata : {
		*(.rodata*)
	} >lowmem

	/DISCARD/ : {
		*(.comment)
		*(.eh_frame)
		*(.rel.eh_frame)
	}
}
