SECTION .text
[BITS 16]
;Generate a memory map at 0x500 to 0x5000 (available memory not used for kernel or bootloader)
memory_map:
	xor eax, eax
	mov di, 0x500
	mov ecx, (0x5000 - 0x500) / 4 ; moving 4bytes at once
	cld
	rep stosd

	mov di, 0x500
	mov edx, 0x534D4150
	xor ebx, ebx
.lp:
	mov eax, 0xE820
	mov ecx, 24

	int 0x15
	jc .done ; Error or finished

	cmp ebx, 0
	je .done ; Finished

	add di, 24
	cmp di, 0x5000
	jb .lp ; Still have buffer space
.done:
	ret
