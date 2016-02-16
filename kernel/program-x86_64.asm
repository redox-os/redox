extern _start_stack
section .text
global _start
_start:
	xchg bx, bx
	mov rdi, rsp
	mov rbp, rsp
	and rsp, 0xFFFFFFFFFFFFFFF0
	call _start_stack
	mov rsp, rbp
	mov rax, 1
	xor rbx, rbx
	int 0x80
