extern _start_stack
section .text
global _start
_start:
	push rsp
	call _start_stack
	add rsp, 4
	mov rax, 1
	xor rbx, rbx
	int 0x80
