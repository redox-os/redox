struc IDTEntry
	.offsetl resw 1
	.selector resw 1
	.ist resb 1
	.attribute resb 1
		.present equ 1 << 7
		.ring1 equ 1 << 5
		.ring2 equ 1 << 6
		.ring3 equ 1 << 5 | 1 << 6
		.ldt32 equ 0x2
		.tssAvailabe64 equ 0x9
		.tssBusy64 equ 0xB
		.callGate64 equ 0xC
		.interrupt64 equ 0xE
		.trap64 equ 0xF
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

    mov rax, gdt.kernel_data
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax

	mov rdi, qword [.entry]
	mov rsi, rsp

		;Stack Align
		mov rbp, rsp
		and rsp, 0xFFFFFFFFFFFFFFF0

		call qword [.handler]

		;Stack Restore
		mov rsp, rbp

	mov rax, gdt.user_data | 3 ;[esp + 44] ;Use new SS as DS
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax

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
		at IDTEntry.attribute, db IDTEntry.present | IDTEntry.interrupt64
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
	at IDTEntry.attribute, db IDTEntry.present | IDTEntry.interrupt64
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
		at IDTEntry.attribute, db IDTEntry.present | IDTEntry.interrupt64
		at IDTEntry.offsetm, dw 0
		at IDTEntry.offseth, dd 0
		at IDTEntry.reserved, dd 0
	iend
%assign i i+1
%endrep
.end:
