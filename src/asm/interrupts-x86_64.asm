struc IDTEntry
	.offsetl resw 1
	.selector resw 1
	.zero1 resb 1
	.attribute resb 1
		.present equ 1 << 7
		.ring.1	equ 1 << 5
		.ring.2 equ 1 << 6
		.ring.3 equ 1 << 5 | 1 << 6
		.task32 equ 0x5
		.interrupt16 equ 0x6
		.trap16 equ 0x7
		.interrupt32 equ 0xE
		.trap32 equ 0xF
	.offsetm resw 1
	.offseth resd 1
	.zero2 resd 1
endstruc

[section .text]
[BITS 64]
interrupts:
.first:
    mov qword [0x100000], qword 0
    jmp qword .handle
.second:
%assign i 1
%rep 255
    mov qword [0x100000], qword i
    jmp qword .handle
%assign i i+1
%endrep
.handle:
    push rax
    push rcx
    push rdx
    push rbx
    push rsp
    push rbp
    push rsi
    push rdi
    push qword [0x100000]
    call qword [.handler]
    ;Put return value in stack for popad
    mov [rsp + 64], rax
    add rsp, 8
    pop rdi
    pop rsi
    pop rbp
    add rsp, 8
    pop rbx
    pop rdx
    pop rcx
    pop rax
    iretq

.handler: dq 0

idtr:
    dw (idt_end - idt) + 1
    dq idt

idt:
%assign i 0
%rep 256	;fill in overrideable functions
	istruc IDTEntry
		at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
		at IDTEntry.selector, dw 0x08
		at IDTEntry.zero1, db 0
		at IDTEntry.attribute, db IDTEntry.present | IDTEntry.interrupt32
		at IDTEntry.offsetm, dw 0
		at IDTEntry.offseth, dd 0
		at IDTEntry.zero2, dd 0
	iend
%assign i i+1
%endrep
idt_end:
