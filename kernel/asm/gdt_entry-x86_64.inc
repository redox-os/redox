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
