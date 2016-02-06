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
