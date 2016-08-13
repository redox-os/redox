struc IDTEntry
    .offsetl resw 1
    .selector resw 1
    .zero resb 1
    .attribute resb 1
    .offseth resw 1
endstruc

SECTION .text
USE32

interrupts:
.first:
    mov [.entry], byte 0
    jmp dword .handle
.second:
%assign i 1
%rep 255
    mov [.entry], byte i
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
    push dword [.entry]

    mov eax, gdt.kernel_data
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax

    	call dword [.handler]

    mov eax, gdt.user_data | 3
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov eax, gdt.user_tls | 3
    mov gs, eax

    add esp, 8 ; Skip interrupt code and reg pointer

    pop eax
    pop ebx
    pop ecx
    pop edx
    pop edi
    pop esi
    pop ebp

    iretd

.handler: dd 0
.entry: dd 0

idtr:
    dw (idt.end - idt) + 1
    dd idt

idt:
%assign i 0

;Below system call
%rep 128
    istruc IDTEntry
        at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
        at IDTEntry.selector, dw gdt.kernel_code
        at IDTEntry.zero, db 0
        at IDTEntry.attribute, db attrib.present | attrib.interrupt32
        at IDTEntry.offseth, dw 0
    iend
%assign i i+1
%endrep

;System call
istruc IDTEntry
    at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
    at IDTEntry.selector, dw gdt.kernel_code
    at IDTEntry.zero, db 0
    at IDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.interrupt32
    at IDTEntry.offseth, dw 0
iend
%assign i i+1

;Above system call
%rep 127
    istruc IDTEntry
        at IDTEntry.offsetl, dw interrupts+(interrupts.second-interrupts.first)*i
        at IDTEntry.selector, dw gdt.kernel_code
        at IDTEntry.zero, db 0
        at IDTEntry.attribute, db attrib.present | attrib.interrupt32
        at IDTEntry.offseth, dw 0
    iend
%assign i i+1
%endrep
.end:
