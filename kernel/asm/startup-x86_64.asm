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

  mov ax, 0x8000
  mov es, ax

  xor edi, edi
  xor eax, eax
  mov ecx, 3 * 4096 / 8 ;PML4, PDP, PD / moves 4 Bytes at once
  cld
  rep stosq

  xor edi, edi
  ;Link first PML4 to PDP
  mov DWORD [es:edi], 0x81000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link first PDP to PD
  mov DWORD [es:edi], 0x82000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link first PD to 1 GB of memory
  mov ebx, 1 << 7 | 1 << 1 | 1
  mov ecx, 512
.setpd:
  mov [es:edi], ebx
  add ebx, 0x200000
  add edi, 8
  loop .setpd

  xor ax, ax
  mov es, ax

  mov edi, 0x80000
  mov cr3, edi

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

    mov rsp, 0x200000 - 128

    ;move kernel image
    mov rdi, kernel_file
    mov rsi, rdi
    add rsi, 0xB000
    mov rcx, (kernel_file.end - kernel_file)
    cld
    rep movsb

    mov rdi, kernel_file.end
    mov rcx, 0xB000
    xor rax, rax
    std
    rep stosb
    cld

    mov rax, gdt.tss
    ltr ax

    ;rust init
    xor rax, rax
    mov [0x100000], rax
    mov eax, [kernel_file + 0x18]
    mov [interrupts.handler], rax
    mov rax, tss
    int 255
.lp:
    sti
    hlt
    jmp .lp

gdtr:
    dw gdt.end + 1  ; size
    dq gdt          ; offset

gdt:
.null equ $ - gdt
    dq 0

.kernel_code equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0         ; base 16:23
    db 0b10011010   ; access byte - code
    db 0b10101111   ; flags/(limit 16:19)
    db 0        ; base 24:31

.kernel_data equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0        ; base 16:23
    db 0b10010010   ; access byte - data
    db 0b10101111   ; flags/(limit 16:19)
    db 0        ; base 24:31

.user_code equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0         ; base 16:23
    db 0b11111010   ; access byte - code
    db 0b10101111   ; flags/(limit 16:19)
    db 0        ; base 24:31

.user_data equ $ - gdt
    dw 0xffff       ; limit 0:15
    dw 0       ; base 0:15
    db 0        ; base 16:23
    db 0b11110010   ; access byte - data
    db 0b10101111   ; flags/(limit 16:19)
    db 0        ; base 24:31

.tss equ $ - gdt
    dw (tss.end-tss) & 0xFFFF         ; limit 0:15
    dw (tss-$$+0x7C00) & 0xFFFF             ; base 0:15
    db ((tss-$$+0x7C00) >> 16) & 0xFF       ; base 16:23
    db 0b11101001                           ; access byte - data
    db 0b01100000 | ((tss.end-tss) >> 16) & 0xF    ; flags/(limit 16:19). flag is set to 32 bit protected mode
    db ((tss-$$+0x7C00) >> 24) & 0xFF       ; base 24:31
    dq 0

.end equ $ - gdt

struc TSS
    .reserved1 resd 1    ;The previous TSS - if we used hardware task switching this would form a linked list.
    .rsp0 resq 1        ;The stack pointer to load when we change to kernel mode.
    .rsp1 resq 1        ;everything below here is unusued now..
    .rsp2 resq 1
    .reserved2 resd 1
    .reserved3 resd 1
    .ist1 resq 1
    .ist2 resq 1
    .ist3 resq 1
    .ist4 resq 1
    .ist5 resq 1
    .ist6 resq 1
    .ist7 resq 1
    .reserved4 resd 1
    .reserved5 resd 1
    .reserved6 resw 1
    .iomap_base resw 1
endstruc

tss:
    istruc TSS
        at TSS.rsp0, dd 0x200000 - 128
        at TSS.iomap_base, dw 0xFFFF
    iend
.end:

%include "asm/interrupts-x86_64.asm"
