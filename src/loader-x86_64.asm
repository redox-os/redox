use16

org 0x7C00

boot: ; dl comes with disk
    ; initialize segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    ; initialize stack
    mov sp, 0x7C00

    mov [disk], dl

    mov si, name
    call print
    call print_line

    mov bh, 0
    mov bl, [disk]
    call print_num
    call print_line

    mov ax, (fs_header - boot)/512
    mov bx, fs_header
    mov cx, (kernel_file.end - fs_header)/512
    xor dx, dx
    call load

    mov si, finished
    call print
    call print_line

    jmp startup

load:
    cmp cx, 64
    jbe .good_size

    pusha
    mov cx, 64
    call load
    popa
    add ax, 64
    add dx, 64*512/16
    sub cx, 64

    jmp load
.good_size:
    mov [DAPACK.addr], ax
    mov [DAPACK.buf], bx
    mov [DAPACK.count], cx
    mov [DAPACK.seg], dx

    mov si, loading
    call print
    call print_line

    mov bx, [DAPACK.addr]
    call print_num

    mov al, '#'
    call print_char

    mov bx, [DAPACK.count]
    call print_num

    call print_line

    mov bx, [DAPACK.seg]
    call print_num

    mov al, ':'
    call print_char

    mov bx, [DAPACK.buf]
    call print_num

    call print_line

    mov dl, [disk]
    mov si, DAPACK
    mov ah, 0x42
    int 0x13
    jc error
    ret

print_char:
    mov ah, 0x0e
    int 0x10
    ret

print_num:
    mov cx, 4
.loop:
    mov al, bh
    shr al, 4
    and al, 0xF

    cmp al, 0xA
    jb .below_a

    add al, 'A' - '0' - 0xA
.below_a:
    add al, '0'

    push cx
    push bx
    call print_char
    pop bx
    pop cx

    shl bx, 4
    loop .loop

    ret

print_line:
    mov si, line
    call print
    ret

print:
.loop:
    lodsb
    or al, al
    jz .done
    call print_char
    jmp .loop
.done:
    ret

error:
  mov si, errored
  call print
  call print_line
.halt:
  cli
  hlt
  jmp .halt

name: db "Redox Loader",0
loading: db "Loading",0
errored: db "Could not read disk",0
finished: db "Finished",0
line: db 13,10,0

disk: db 0

DAPACK:
        db	0x10
        db	0
.count: dw	0	; int 13 resets this to # of blocks actually read/written
.buf:   dw	0 ; memory buffer destination address (0:7c00)
.seg:   dw	0	; in memory page zero
.addr:  dd	0	; put the lba to read in this spot
        dd	0	; more storage bytes only for big lba's ( > 4 bytes )

times 510-($-$$) db 0
db 0x55
db 0xaa

fs_header:
.signature:
    db "REDOXFS",0
.version:
    dd 0xFFFFFFFF
.name:
    db "Root Filesystem",0
align 256, db 0
.extents:
    dq (fs_root_node_list - boot)/512
    dq (fs_root_node_list.end - fs_root_node_list)

    align 512, db 0
.end:

startup:
  ; a20
  in al, 0x92
  or al, 2
  out 0x92, al

  call memory_map

  call vesa

  call initialize.fpu
  call initialize.sse
  call initialize.pit
  call initialize.pic

  ; load protected mode GDT and IDT
  cli

  mov edi, 0x1000
  mov cr3, edi
  xor eax, eax
  mov ecx, 3 * 1024 ;PML4, PDP, PD
  rep stosd
  mov edi, cr3

  ;Link first PML4 to PDP
  mov DWORD [edi], 0x2000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link first PDP to PD
  mov DWORD [edi], 0x3000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link first PD to 1 GB of memory
  mov ebx, 1 << 7 | 1 << 1 | 1
  mov ecx, 512
.setpd:
  mov [edi], ebx
  add ebx, 0x200000
  add edi, 8
  loop .setpd

  mov eax, cr4
  or eax, 1 << 5 | 1 << 4
  mov cr4, eax

  lgdt [gdtr]
  lidt [idtr]

  mov ecx, 0xC0000080               ; Read from the EFER MSR.
  rdmsr
  or eax, 0x00000100                ; Set the LME bit.
  wrmsr

  mov ebx, cr0                      ; Activate long mode -
  or ebx, 0x80000001                 ; - by enabling paging and protection simultaneously.
  mov cr0, ebx

  ; far jump to load CS with 32 bit segment
  jmp 0x08:long_mode

%include "asm/memory_map.asm"
%include "asm/vesa.asm"
%include "asm/initialize.asm"

long_mode:
    use64
    ; load all the other segments with 32 bit data segments
    mov rax, 0x10
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax
    mov ss, rax
    ; set up stack
    mov rsp, 0x200000

    ;rust init
    xor rax, rax
    mov eax, [kernel_file + 0x18]
    mov [interrupts.handler], rax

    mov rdi, kernel_file
    mov rsi, rdi
    add rsi, 0xB000
    mov rcx, (kernel_file.font - kernel_file)
    cld
    rep movsb

    mov rdi, kernel_file.font
    mov rcx, 0xB000
    xor rax, rax
    std
    rep stosb

    cld

    mov rax, kernel_file.font
    int 255
.lp:
    sti
    hlt
    jmp .lp

gdtr:
    dw (gdt_end - gdt) + 1  ; size
    dq gdt                  ; offset

gdt:
    ; null entry
    dq 0
    ; code entry
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0         ; base 16:23
    db 0b10011010   ; access byte - code
    db 0b00101111   ; flags/(limit 16:19)
    db 0        ; base 24:31
    ; data entry
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0        ; base 16:23
    db 0b10010010   ; access byte - data
    db 0b00101111   ; flags/(limit 16:19)
    db 0        ; base 24:31
gdt_end:

%include "asm/interrupts-x86_64.asm"

times (0xC000-0x1000)-0x7C00-($-$$) db 0

kernel_file:
  incbin "kernel.bin"
  align 512, db 0

.font:
  incbin "ui/unifont.font"
  align 512, db 0
.end:

fs_root_node_list:
%macro file 2+
    fs_node.%1:
    .name:
        db %2,0

        align 256, db 0

    .extents:
        dq (fs_data.%1 - boot)/512
        dq (fs_data.%1.end - fs_data.%1)

        align 512, db 0
    .end:
%endmacro

%include "filesystem.gen"

%unmacro file 2+
fs_root_node_list.end:

%macro file 2+
fs_data.%1:
    incbin %2
.end:
    align 512, db 0
%endmacro

%include "filesystem.gen"

%unmacro file 2+
