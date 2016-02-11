%include "asm/startup-common.asm"

startup_arch:
high equ 0xC0000000
pdp_table equ 0x70000
pd_table equ pdp_table + 0x1000

; setting up Page Tables
    cli

    ; clearing page tables
    xor eax, eax
    mov edi, pdp_table
    mov ecx, 2 * 4096 / 4 ;PDP, PD / moves 4 Bytes at once
    cld
    rep stosd

    ; Mapping first and 4th GiB of virt. mem to first Gib of phys. mem
    mov edi, pdp_table
    ;Link first PDPE to PD
    mov DWORD [edi + 0 * 8], pd_table | 1 ; maps first GiB
    mov DWORD [edi + 3 * 8], pd_table | 1 ; maps last GiB
    add edi, 0x1000
    ;Link all PDEs(each 2MiB) to the first GiB
    mov ebx, 1 << 7 | 1 << 1 | 1 ; large page, writable, present, don't enable global pages yet, this is only a temporary mapping
    mov ecx, 512
.setpd:
    mov [edi], ebx
    add ebx, 0x200000
    add edi, 8
    loop .setpd

    ; cr3 holds pointer to PDP
    mov eax, pdp_table
    mov cr3, eax

    ; enable Page Global Extension and Page Address Extension and Page Size Extension and Page
    ; PAE needs at least i686(P6)
    mov eax, cr4
    or eax, 1 << 7 | 1 << 5 | 1 << 4
    mov cr4, eax

    ; load low protected mode GDT
    lgdt [gdtr]

    ; set paging and protected mode bit of cr0
    mov eax, cr0
    or eax, 1 << 31 | 1
    ;or eax, 1
    mov cr0, eax

    ; far jump to higher memory, loads CS with 32 bit segment
    jmp DWORD gdt.kernel_code:high + protected_mode

USE32
protected_mode:

    ; load all the other segments with 32 bit data segments
    mov eax, gdt.kernel_data
    mov ds, eax
    mov es, eax
    mov fs, eax
    mov gs, eax
    mov ss, eax

    ; setup stack
    mov esp, 0x200000 - 128 + high

    ; clear identity mapping
    mov edi, pdp_table
    mov DWORD [edi], 0

    ; reload TLB
    mov eax, cr3
    mov cr3, eax

    ; load high protected mode GDT and IDT
    lidt [high + idtr]
    lgdt [high + gdtr_high]

    ; load task register
    mov eax, gdt.tss
    ltr ax

    ;rust init
    mov eax, [high + kernel_base + 0x18]
    mov [high + interrupts.handler], eax
    mov eax, high + tss
    int 0xFF
.lp:
    cli
    hlt
    jmp .lp

gdtr:
    dw gdt.end + 1  ; size
    dd gdt          ; offset

gdtr_high:
    dw gdt.end + 1  ; size
    dd high + gdt   ; offset

gdt:
.null equ $ - gdt
    dq 0

.kernel_code equ $ - gdt
    istruc GDTEntry
        at GDTEntry.limitl, dw 0xFFFF
        at GDTEntry.basel, dw 0
        at GDTEntry.basem, db 0
        at GDTEntry.attribute, db attrib.present | attrib.user | attrib.code | attrib.readable
        at GDTEntry.flags__limith, db 0xFF | flags.granularity | flags.default_operand_size
        at GDTEntry.baseh, db 0
    iend

.kernel_data equ $ - gdt
    istruc GDTEntry
        at GDTEntry.limitl, dw 0xFFFF
        at GDTEntry.basel, dw 0
        at GDTEntry.basem, db 0
        at GDTEntry.attribute, db attrib.present | attrib.user | attrib.writable
        at GDTEntry.flags__limith, db 0xFF | flags.granularity | flags.default_operand_size
        at GDTEntry.baseh, db 0
    iend

.user_code equ $ - gdt
    istruc GDTEntry
        at GDTEntry.limitl, dw 0xFFFF
        at GDTEntry.basel, dw 0
        at GDTEntry.basem, db 0
        at GDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.user | attrib.code | attrib.readable
        at GDTEntry.flags__limith, db 0xFF | flags.granularity | flags.default_operand_size
        at GDTEntry.baseh, db 0
    iend

.user_data equ $ - gdt
    istruc GDTEntry
        at GDTEntry.limitl, dw 0xFFFF
        at GDTEntry.basel, dw 0
        at GDTEntry.basem, db 0
        at GDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.user | attrib.writable
        at GDTEntry.flags__limith, db 0xFF | flags.granularity | flags.default_operand_size
        at GDTEntry.baseh, db 0
    iend

.tss equ $ - gdt
    istruc GDTEntry
        at GDTEntry.limitl, dw (tss.end - tss) & 0xFFFF
        at GDTEntry.basel, dw (tss - $$ + 0x7C00) & 0xFFFF
        at GDTEntry.basem, db ((tss - $$ + 0x7C00) >> 16) & 0xFF
        at GDTEntry.attribute, db attrib.present | attrib.ring3 | attrib.tssAvailabe32
        at GDTEntry.flags__limith, db ((tss.end - tss) >> 16) & 0xF
        at GDTEntry.baseh, db ((tss - $$ + 0x7C00) >> 24) & 0xFF
    iend
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
        at TSS.esp0, dd 0x200000 - 128 + high
        at TSS.ss0, dd gdt.kernel_data
        at TSS.iomap_base, dw 0xFFFF
    iend
.end:

%include "asm/interrupts-i386.asm"
