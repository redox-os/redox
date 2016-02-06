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
    mov cx, kernel_file.length_sectors / buffer_size_sectors

    mov ax, (kernel_file - boot) / 512
    mov edi, kernel_base
    cld
.lp:
xchg bx, bx
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
    xchg bx, bx

    ; load the part of the kernel that does not fill the buffer completely
    mov cx, kernel_file.length_sectors % buffer_size_sectors
    mov bx, startup_end
    mov dx, 0x0
    call load

    xchg bx, bx
    ; moving remnants of kernel
    call unreal

    xchg bx, bx
    mov esi, startup_end
    mov ecx, (kernel_file.length_sectors % buffer_size_bytes) / 4
    a32 rep movsd

    xchg bx, bx
    call memory_map

    call vesa

    call initialize.fpu
    call initialize.sse
    call initialize.pit
    call initialize.pic

    xchg bx, bx
    jmp startup_arch

%include "asm/unreal.asm"
%include "asm/memory_map.asm"
%include "asm/vesa.asm"
%include "asm/initialize.asm"

%include "asm/gdt_entry.inc"

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
        at GDTEntry.access,        db GDTEntry.present | GDTEntry.user | GDTEntry.data_writable
        at GDTEntry.flags__limith, db 0xFF | GDTEntry.granularity | GDTEntry.default_operand_size
        at GDTEntry.baseh,         db 0x0

    iend
.end equ $ - unreal_gdt
