struc IDTEntry
    .offsetl resw 1
    .selector resw 1
    .zero resb 1
    .attribute resb 1
        .present equ 1 << 7
        .ring1 equ 1 << 5
        .ring2 equ 1 << 6
        .ring3 equ 1 << 5 | 1 << 6
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
    mov [0x100000], byte 0
    jmp dword .handle
.second:
%assign i 1
%rep 255
    mov [0x100000], byte i
    jmp dword .handle
%assign i i+1
%endrep
.handle:

    push ebp
    push esi
    push edi
    push edx
    push ecx
    push ebx
    push eax
    push esp
    push dword [0x100000]

    mov eax, gdt.kernel_data
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax

    call [.handler]

    add esp, 8 ;Skip interrupt and reg pointer

    mov eax, gdt.user_data ;[esp + 44] ;Use new SS as DS
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax

    pop eax
    pop ebx
    pop ecx
    pop edx
    pop edi
    pop esi
    pop ebp

    xchg bx, bx

    iretd

.handler: dd 0

idtr:
    dw (idt_end - idt) + 1
    dd idt

idt:
%assign i 0
%rep 256	;fill in overrideable functions
	istruc IDTEntry
		at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
		at IDTEntry.selector, dw gdt.kernel_code
        at IDTEntry.zero, db 0
		at IDTEntry.attribute, db IDTEntry.ring3 | IDTEntry.present | IDTEntry.interrupt32 ;TODO: Use ring 3 only for 0x80 and 0xFF, disable 0xFF after boot
        at IDTEntry.offseth, dw 0
	iend
%assign i i+1
%endrep
idt_end:
