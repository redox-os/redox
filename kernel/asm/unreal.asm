SECTION .text
USE16

; switch to unreal mode; ds and es can address up to 4GiB
unreal:
    cli

    lgdt [unreal_gdtr]

    push es
    push ds

    mov  eax, cr0          ; switch to pmode by
    or al,1                ; set pmode bit
    mov  cr0, eax

    jmp $+2

; http://wiki.osdev.org/Babystep7
; When this register given a "selector", a "segment descriptor cache register"
; is filled with the descriptor values, including the size (or limit). After
; the switch back to real mode, these values are not modified, regardless of
; what value is in the 16-bit segment register. So the 64k limit is no longer
; valid and 32-bit offsets can be used with the real-mode addressing rules
    mov bx, unreal_gdt.data
    mov es, bx
    mov ds, bx

    and al,0xFE            ; back to realmode
    mov  cr0, eax          ; by toggling bit again

    pop ds
    pop es
    sti
    ret


unreal_gdtr:
    dw unreal_gdt.end + 1  ; size
    dd unreal_gdt          ; offset

unreal_gdt:
.null equ $ - unreal_gdt
    dq 0
.data equ $ - unreal_gdt
    istruc GDTEntry
        at GDTEntry.limitl,        dw 0xFFFF
        at GDTEntry.basel,         dw 0x0
        at GDTEntry.basem,         db 0x0
        at GDTEntry.attribute,        db attrib.present | attrib.user | attrib.writable
        at GDTEntry.flags__limith, db 0xFF | flags.granularity | flags.default_operand_size
        at GDTEntry.baseh,         db 0x0
    iend
.end equ $ - unreal_gdt
