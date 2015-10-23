extern _start_stack
section .text
global _start
_start:
	push esp
	call _start_stack
	add esp, 4
	mov eax, 1
	xor ebx, ebx
	int 0x80
