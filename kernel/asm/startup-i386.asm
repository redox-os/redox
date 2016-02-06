%include "asm/startup-common.asm"

startup_arch:
    ; load protected mode GDT and IDT
    cli
    lgdt [gdtr]
    lidt [idtr]
    ; set protected mode bit of cr0
    mov eax, cr0
    or eax, 1
    mov cr0, eax

    ; far jump to load CS with 32 bit segment
    jmp gdt.kernel_code:protected_mode

USE32
protected_mode:

    ; load all the other segments with 32 bit data segments
    mov eax, gdt.kernel_data
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    mov esp, 0x200000 - 128

    mov eax, gdt.tss
    ltr ax

    ;rust init
    xor eax, eax
    mov [0x100000], eax
    mov eax, [kernel_file + 0x18]
    mov [interrupts.handler], eax
    mov eax, tss
    int 255
.lp:
    sti
    hlt
    jmp .lp

gdtr:
    dw gdt.end + 1  ; size
    dd gdt          ; offset

gdt:
.null equ $ - gdt
    dq 0

.kernel_code equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0b10011010   ; access byte - code
    db 0xdf         ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db 0x00         ; base 24:31

.kernel_data equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0b10010010   ; access byte - data
    db 0xdf         ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db 0x00         ; base 24:31

.user_code equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0b11111010   ; access byte - code
    db 0xdf         ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db 0x00         ; base 24:31

.user_data equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0x0000       ; base 0:15
    db 0x00         ; base 16:23
    db 0b11110010   ; access byte - data
    db 0xdf         ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db 0x00         ; base 24:31

.tss equ $ - gdt
    dw (tss.end-tss) & 0xFFFF         ; limit 0:15
    dw (tss-$$+0x7C00) & 0xFFFF             ; base 0:15
    db ((tss-$$+0x7C00) >> 16) & 0xFF       ; base 16:23
    db 0b11101001                           ; access byte - data
    db 0x40 | ((tss.end-tss) >> 16) & 0xF    ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db ((tss-$$+0x7C00) >> 24) & 0xFF       ; base 24:31

.end equ $ - gdt

struc TSS
    .prev_tss resd 1    ;The previous TSS - if we used hardware task switching this would form a linked list.
    .esp0 resd 1        ;The stack pointer to load when we change to kernel mode.
    .ss0 resd 1         ;The stack segment to load when we change to kernel mode.
    .esp1 resd 1        ;everything below here is unusued now..
    .ss1 resd 1
    .esp2 resd 1
    .ss2 resd 1
    .cr3 resd 1
    .eip resd 1
    .eflags resd 1
    .eax resd 1
    .ecx resd 1
    .edx resd 1
    .ebx resd 1
    .esp resd 1
    .ebp resd 1
    .esi resd 1
    .edi resd 1
    .es resd 1
    .cs resd 1
    .ss resd 1
    .ds resd 1
    .fs resd 1
    .gs resd 1
    .ldt resd 1
    .trap resw 1
    .iomap_base resw 1
endstruc

tss:
    istruc TSS
        at TSS.esp0, dd 0x200000 - 128
        at TSS.ss0, dd gdt.kernel_data
        at TSS.iomap_base, dw 0xFFFF
    iend
.end:

%include "asm/interrupts-i386.asm"
