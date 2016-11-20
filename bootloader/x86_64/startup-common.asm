SECTION .text
USE16

startup:
    ; enable A20-Line via IO-Port 92, might not work on all motherboards
    in al, 0x92
    or al, 2
    out 0x92, al

; loading kernel to 1MiB
; move part of kernel to startup_end via bootsector#load and then copy it up
; repeat until all of the kernel is loaded

; buffersize in multiple of sectors (512 Bytes)
; min 1
; max (0x70000 - startup_end) / 512
buffer_size_sectors equ 127
; buffer size in Bytes
buffer_size_bytes equ buffer_size_sectors * 512

kernel_base equ 0x100000

    ; how often do we need to call load and move memory
    mov ecx, kernel_file.length_sectors / buffer_size_sectors

    mov eax, (kernel_file - boot) / 512
    mov edi, kernel_base
    cld
.lp:
    ; saving counter
    push cx

        ; populating buffer
        mov cx, buffer_size_sectors
        mov bx, kernel_file
        mov dx, 0x0

        push edi
        push eax
        call load

        ; moving buffer
        call unreal
        pop eax
        pop edi

        mov esi, kernel_file
        mov ecx, buffer_size_bytes / 4
        a32 rep movsd

        ; preparing next iteration
        add eax, buffer_size_sectors

    pop cx
    loop .lp

    ; load the part of the kernel that does not fill the buffer completely
    mov cx, kernel_file.length_sectors % buffer_size_sectors
    test cx, cx
    jz finished_loading ; if cx = 0 => skip

    mov bx, kernel_file
    mov dx, 0x0
    call load

    ; moving remnants of kernel
    call unreal

    mov esi, kernel_file
    mov ecx, (kernel_file.length_sectors % buffer_size_bytes) / 4
    a32 rep movsd
finished_loading:
    call memory_map

    call vesa

    mov si, init_fpu_msg
    call printrm
    call initialize.fpu

    mov si, init_sse_msg
    call printrm
    call initialize.sse

    mov si, init_pit_msg
    call printrm
    call initialize.pit

    mov si, init_pic_msg
    call printrm
    call initialize.pic

    mov si, startup_arch_msg
    call printrm

    jmp startup_arch

%include "config.asm"
%include "descriptor_flags.inc"
%include "gdt_entry.inc"
%include "unreal.asm"
%include "memory_map.asm"
%include "vesa.asm"
%include "initialize.asm"

init_fpu_msg: db "Init FPU",13,10,0
init_sse_msg: db "Init SSE",13,10,0
init_pit_msg: db "Init PIT",13,10,0
init_pic_msg: db "Init PIC",13,10,0
startup_arch_msg: db "Startup Arch",13,10,0
