struc IDTEntry
	.offsetl resw 1
	.selector resw 1
	.ist resb 1
	.attribute resb 1
	.offsetm resw 1
	.offseth resd 1
	.reserved resd 1
endstruc

SECTION .text
USE64
interrupts:
.first:
	mov [.entry], byte 0
    jmp qword .handle
.second:
%assign i 1
%rep 255
	mov [.entry], byte i
    jmp qword .handle
%assign i i+1
%endrep
.handle:
	push rbp
	push r15
	push r14
	push r13
	push r12
	push r11
	push r10
	push r9
	push r8
	push rsi
	push rdi
	push rdx
	push rcx
	push rbx
	push rax

	mov rsi, rsp
	push rsi
	mov rdi, qword [.entry]
	push rdi

    mov rax, gdt.kernel_data
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax

		call qword [.handler]

	mov rax, gdt.user_data | 3 ;[esp + 44] ;Use new SS as DS
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax

	add rsp, 16 ; Skip interrupt code and reg pointer

	pop rax
	pop rbx
	pop rcx
	pop rdx
	pop rdi
	pop rsi
	pop r8
	pop r9
	pop r10
	pop r11
	pop r12
	pop r13
	pop r14
	pop r15
	pop rbp

    iretq

.handler: dq 0
.entry: dq 0

idtr:
    dw (idt.end - idt) + 1
    dq idt

idt:
%assign i 0

;Below syscall
%rep 128
	istruc IDTEntry
		at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
		at IDTEntry.selector, dw gdt.kernel_code
		at IDTEntry.ist, db 0
		at IDTEntry.attribute, db attrib.present | attrib.interrupt64
		at IDTEntry.offsetm, dw 0
		at IDTEntry.offseth, dd 0
		at IDTEntry.reserved, dd 0
	iend
%assign i i+1
%endrep

;Syscall
istruc IDTEntry
	at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
	at IDTEntry.selector, dw gdt.kernel_code
	at IDTEntry.ist, db 0
	at IDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.interrupt64
	at IDTEntry.offsetm, dw 0
	at IDTEntry.offseth, dd 0
	at IDTEntry.reserved, dd 0
iend
%assign i i+1

;Above syscall
%rep 127
	istruc IDTEntry
		at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
		at IDTEntry.selector, dw gdt.kernel_code
		at IDTEntry.ist, db 0
		at IDTEntry.attribute, db attrib.present | attrib.interrupt64
		at IDTEntry.offsetm, dw 0
		at IDTEntry.offseth, dd 0
		at IDTEntry.reserved, dd 0
	iend
%assign i i+1
%endrep
.end:
