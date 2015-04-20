struc IDTEntry
	.offsetl resw 1
	.selector resw 1
	.zero resb 1
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
	.offseth resw 1
endstruc

[section .text]
[BITS 32]
interrupts:
.first:
	mov [0x200000], byte 0
	jmp dword .handle
.second:
%assign i 1
%rep 255
	mov [0x200000], byte i
	jmp dword .handle
%assign i i+1
%endrep
.handle:
	pushad
	
	mov al, [0x200000]
	cmp al, 0x20
	je .ignore
	
	call [.callback]
.ignore:
    mov al, [0x200000]
    
    cmp al, 0x20
    jb .not_irq
    
    cmp al, 0x30
    jae .not_irq
    
    cmp al, 0x28
    jb .not_slave
    
    mov dx, 0xA0
    mov al, 0x20
    out dx, al
.not_slave:
    mov dx, 0x20
    mov al, 0x20
    out dx, al
.not_irq:
	popad
	iretd
	
.callback: dq .ignore

idtr:
    dw (idt_end - idt) + 1
    dd idt
    
idt:
%assign i 0
%rep 256	;fill in overrideable functions
	istruc IDTEntry
		at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
		at IDTEntry.selector, dw 0x08
		at IDTEntry.attribute, db IDTEntry.present | IDTEntry.interrupt32
	iend
%assign i i+1
%endrep
idt_end:
