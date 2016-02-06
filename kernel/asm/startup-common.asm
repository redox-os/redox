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
; max (0x80000 - startup_end) / 512
buffer_size_sectors equ 64
; buffer size in Bytes
buffer_size_bytes equ buffer_size_sectors * 512

kernel_base equ 0x100000

    ; how often do we need to call load and move memory
    mov ecx, kernel_file.length_sectors / buffer_size_sectors

    mov ax, (kernel_file - boot) / 512
    mov edi, kernel_base
    cld
.lp:
    ; saving counter
    push cx

        ; populating buffer
        mov cx, buffer_size_sectors
        mov bx, startup_end
        mov dx, 0x0

; TODO test if other registers are affected
        push ax
        call load

        ; moving buffer
        call unreal
        pop ax

        mov esi, startup_end
        mov ecx, buffer_size_bytes / 4
        a32 rep movsd

        ; preparing next iteration
        add ax, buffer_size_sectors

    pop cx
    loop .lp

    ; load the part of the kernel that does not fill the buffer completely
    mov cx, kernel_file.length_sectors % buffer_size_sectors
    test cx, cx
    jz finished_loading ; if cx = 0 => skip

    mov bx, startup_end
    mov dx, 0x0
    call load

    ; moving remnants of kernel
    call unreal

    mov esi, startup_end
    mov ecx, (kernel_file.length_sectors % buffer_size_bytes) / 4
    a32 rep movsd
finished_loading:


    call memory_map

    call vesa

    call initialize.fpu
    call initialize.sse
    call initialize.pit
    call initialize.pic

    jmp startup_arch

%include "asm/gdt_entry.inc"
%include "asm/unreal.asm"
%include "asm/memory_map.asm"
%include "asm/vesa.asm"
%include "asm/initialize.asm"
