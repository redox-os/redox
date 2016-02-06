SECTION .text
[BITS 16]
;Generate a memory map at 0x500 to 0x5000 (available memory not used for kernel or bootloader)
memory_map:
.start  equ 0x0500
.end    equ 0x5000
.length equ .end - .start

    xor eax, eax
    mov di, .start
    mov ecx, .length / 4 ; moving 4 Bytes at once
    cld
    rep stosd

    mov di, .start
    mov edx, 0x534D4150
    xor ebx, ebx
.lp:
    mov eax, 0xE820
    mov ecx, 24

    int 0x15
    jc .done ; Error or finished

    cmp ebx, 0
    je .done ; Finished

    add di, 24
    cmp di, .end
    jb .lp ; Still have buffer space
.done:
    ret
