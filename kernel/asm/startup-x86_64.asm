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

  ; setting up Page Tables
  ; Identity Mapping first 512GB
  cli
  mov ax, 0x8000
  mov es, ax

  xor edi, edi
  xor eax, eax
  mov ecx, 3 * 4096 / 4 ;PML4, PDP, PD / moves 4 Bytes at once
  cld
  rep stosd

  xor edi, edi
  ;Link first PML4 to PDP
  mov DWORD [es:edi], 0x81000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link first PDP to PD
  mov DWORD [es:edi], 0x82000 | 1 << 1 | 1
  add edi, 0x1000
  ;Link all PD's (512 per PDP) to 1 GB of memory
  mov ebx, 1 << 7 | 1 << 1 | 1
  mov ecx, 512
.setpd:
  mov [es:edi], ebx
  add ebx, 0x200000
  add edi, 8
  loop .setpd

  xor ax, ax
  mov es, ax

  ;cr3 holds pointer to PML4
  mov edi, 0x80000
  mov cr3, edi

  ;enable Page Address Extension and Page Size Extension
  mov eax, cr4
  or eax, 1 << 5 | 1 << 4
  mov cr4, eax

  ; load protected mode GDT
  lgdt [gdtr]

  mov ecx, 0xC0000080               ; Read from the EFER MSR.
  rdmsr
  or eax, 0x00000100                ; Set the Long-Mode-Enable bit.
  wrmsr

  ;enabling paging and protection simultaneously
  mov ebx, cr0
  or ebx, 0x80000001                ;Bit 31: Paging, Bit 0: Protected Mode
  mov cr0, ebx

  ; far jump to enable Long Mode and load CS with 64 bit code segment
  jmp gdt.kernel_code:long_mode

%include "asm/memory_map.asm"
%include "asm/vesa.asm"
%include "asm/initialize.asm"

use64
long_mode:

    ; load all the other segments with 64 bit data segments
    mov rax, gdt.kernel_data
    mov ds, rax
    mov es, rax
    mov fs, rax
    mov gs, rax
    mov ss, rax

    ; load long mode IDT
    lidt [idtr]

    ; provide stack
    mov rsp, 0x200000 - 128

    mov rax, gdt.tss
    ltr ax

    ;rust init
    xor rax, rax
    mov [0x100000], rax
    mov eax, [kernel_file + 0x18]
    mov [interrupts.handler], rax
    mov rax, tss
    int 0xFF
.lp:
    sti
    hlt
    jmp .lp

struc GDTEntry
    .limitl resw 1
    .basel resw 1
    .basem resb 1
    .access resb 1
        ;both
        .present equ 1 << 7
        .ring1 equ 1 << 5
        .ring2 equ 1 << 6
        .ring3 equ 1 << 5 | 1 << 6
        .user equ 1 << 4
        ;user
        .code equ 1 << 3
        .code_conforming equ 1 << 2
        .code_readable equ 1 << 1
        .data_expand_down equ 1 << 2
        .data_writable equ 1 << 1
        .accessed equ 1 << 0
        ;system
        .ldt32 equ 0x2
        .tssAvailabe64 equ 0x9
        .tssBusy64 equ 0xB
        .callGate64 equ 0xC
        .interrupt64 equ 0xE
        .trap64 equ 0xF
    .flags__limith resb 1
        ;both
        .granularity equ 1 << 7
        .available equ 1 << 4
        ;user
        .default_operand_size equ 1 << 6
        .code_long_mode equ 1 << 5
        .data_reserved equ 1 << 5
    .baseh resb 1
endstruc

gdtr:
    dw gdt.end + 1  ; size
    dq gdt          ; offset

gdt:
.null equ $ - gdt
    dq 0

.kernel_code equ $ - gdt
istruc GDTEntry
    at GDTEntry.limitl, dw 0
    at GDTEntry.basel, dw 0
    at GDTEntry.basem, db 0
    at GDTEntry.access, db GDTEntry.present | GDTEntry.user | GDTEntry.code
    at GDTEntry.flags__limith, db GDTEntry.code_long_mode
    at GDTEntry.baseh, db 0
iend

.kernel_data equ $ - gdt
istruc GDTEntry
    at GDTEntry.limitl, dw 0
    at GDTEntry.basel, dw 0
    at GDTEntry.basem, db 0
; AMD System Programming Manual states that the writeable bit is ignored in long mode, but ss can not be set to this descriptor without it
    at GDTEntry.access, db GDTEntry.present | GDTEntry.user | GDTEntry.data_writable
    at GDTEntry.flags__limith, db 0
    at GDTEntry.baseh, db 0
iend

.user_code equ $ - gdt
istruc GDTEntry
    at GDTEntry.limitl, dw 0
    at GDTEntry.basel, dw 0
    at GDTEntry.basem, db 0
    at GDTEntry.access, db GDTEntry.present | GDTEntry.ring3 | GDTEntry.user | GDTEntry.code
    at GDTEntry.flags__limith, db GDTEntry.code_long_mode
    at GDTEntry.baseh, db 0
iend

.user_data equ $ - gdt
istruc GDTEntry
    at GDTEntry.limitl, dw 0
    at GDTEntry.basel, dw 0
    at GDTEntry.basem, db 0
; AMD System Programming Manual states that the writeable bit is ignored in long mode, but ss can not be set to this descriptor without it
    at GDTEntry.access, db GDTEntry.present | GDTEntry.ring3 | GDTEntry.user | GDTEntry.data_writable
    at GDTEntry.flags__limith, db 0
    at GDTEntry.baseh, db 0
iend

.tss equ $ - gdt
istruc GDTEntry
    at GDTEntry.limitl, dw (tss.end-tss) & 0xFFFF
    at GDTEntry.basel, dw (tss-$$+0x7C00) & 0xFFFF
    at GDTEntry.basem, db ((tss-$$+0x7C00) >> 16) & 0xFF
    at GDTEntry.access, db GDTEntry.present | GDTEntry.ring3 | GDTEntry.tssAvailabe64
    at GDTEntry.flags__limith, db ((tss.end-tss) >> 16) & 0xF
    at GDTEntry.baseh, db ((tss-$$+0x7C00) >> 24) & 0xFF
iend
dq 0 ;tss descriptors are extended to 16 Bytes

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
