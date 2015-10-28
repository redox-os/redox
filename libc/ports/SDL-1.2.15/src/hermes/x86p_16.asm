;
; x86 format converters for HERMES
; Copyright (c) 1998 Glenn Fielder (gaffer@gaffer.org)
; This source code is licensed under the GNU LGPL
; 
; Please refer to the file COPYING.LIB contained in the distribution for
; licensing conditions		
; 
; Routines adjusted for Hermes by Christian Nentwich (brn@eleet.mcb.at)
; Used with permission.
; 

BITS 32

%include "common.inc"

SDL_FUNC _ConvertX86p16_16BGR565
SDL_FUNC _ConvertX86p16_16RGB555
SDL_FUNC _ConvertX86p16_16BGR555
SDL_FUNC _ConvertX86p16_8RGB332

EXTERN _ConvertX86

SECTION .text

_ConvertX86p16_16BGR565:

    ; check short
    cmp ecx,BYTE 16
    ja .L3


.L1: ; short loop
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    and ebx,11111100000b
    shl edx,11
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; head
    mov eax,edi
    and eax,BYTE 11b
    jz .L4
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    and ebx,11111100000b
    shl edx,11
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx

.L4: ; save count
    push ecx

    ; unroll twice
    shr ecx,1
    
    ; point arrays to end
    lea esi,[esi+ecx*4]
    lea edi,[edi+ecx*4]

    ; negative counter 
    neg ecx
    jmp SHORT .L6
                              
.L5:    mov [edi+ecx*4-4],eax
.L6:    mov eax,[esi+ecx*4]

        mov ebx,[esi+ecx*4]
        and eax,07E007E0h         

        mov edx,[esi+ecx*4]
        and ebx,0F800F800h

        shr ebx,11
        and edx,001F001Fh

        shl edx,11
        add eax,ebx

        add eax,edx                 
        inc ecx

        jnz .L5                 
         
    mov [edi+ecx*4-4],eax

    ; tail
    pop ecx
    and ecx,BYTE 1
    jz .L7
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    and ebx,11111100000b
    shl edx,11
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2

.L7:
    retn






_ConvertX86p16_16RGB555:

    ; check short
    cmp ecx,BYTE 32
    ja .L3


.L1: ; short loop
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    shr ebx,1
    and ebx,     0111111111100000b
    and eax,BYTE 0000000000011111b
    add eax,ebx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; head
    mov eax,edi
    and eax,BYTE 11b
    jz .L4
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    shr ebx,1
    and ebx,     0111111111100000b
    and eax,BYTE 0000000000011111b
    add eax,ebx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx

.L4: ; save ebp
    push ebp

    ; save count
    push ecx

    ; unroll four times
    shr ecx,2
    
    ; point arrays to end
    lea esi,[esi+ecx*8]
    lea edi,[edi+ecx*8]

    ; negative counter 
    xor ebp,ebp
    sub ebp,ecx

.L5:    mov eax,[esi+ebp*8]        ; agi?
        mov ecx,[esi+ebp*8+4]
       
        mov ebx,eax
        mov edx,ecx

        and eax,0FFC0FFC0h
        and ecx,0FFC0FFC0h

        shr eax,1
        and ebx,001F001Fh

        shr ecx,1
        and edx,001F001Fh

        add eax,ebx
        add ecx,edx

        mov [edi+ebp*8],eax
        mov [edi+ebp*8+4],ecx

        inc ebp
        jnz .L5                 

    ; tail
    pop ecx
.L6: and ecx,BYTE 11b
    jz .L7
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    shr ebx,1
    and ebx,     0111111111100000b
    and eax,BYTE 0000000000011111b
    add eax,ebx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx
    jmp SHORT .L6

.L7: pop ebp
    retn






_ConvertX86p16_16BGR555:

    ; check short
    cmp ecx,BYTE 16
    ja .L3

	
.L1: ; short loop
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    shr ebx,1
    and ebx,1111100000b
    shl edx,10
    and edx,0111110000000000b
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; head
    mov eax,edi
    and eax,BYTE 11b
    jz .L4
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    shr ebx,1
    and ebx,1111100000b
    shl edx,10
    and edx,0111110000000000b
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2
    dec ecx

.L4: ; save count
    push ecx

    ; unroll twice
    shr ecx,1
    
    ; point arrays to end
    lea esi,[esi+ecx*4]
    lea edi,[edi+ecx*4]

    ; negative counter 
    neg ecx
    jmp SHORT .L6
                              
.L5:     mov [edi+ecx*4-4],eax
.L6:     mov eax,[esi+ecx*4]

        shr eax,1
        mov ebx,[esi+ecx*4]
        
        and eax,03E003E0h         
        mov edx,[esi+ecx*4]

        and ebx,0F800F800h

        shr ebx,11
        and edx,001F001Fh

        shl edx,10
        add eax,ebx

        add eax,edx                 
        inc ecx

        jnz .L5                 
         
    mov [edi+ecx*4-4],eax

    ; tail
    pop ecx
    and ecx,BYTE 1
    jz .L7
    mov al,[esi]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    shr eax,11
    and eax,BYTE 11111b
    shr ebx,1
    and ebx,1111100000b
    shl edx,10
    and edx,0111110000000000b
    add eax,ebx
    add eax,edx
    mov [edi],al
    mov [edi+1],ah
    add esi,BYTE 2
    add edi,BYTE 2

.L7:
    retn






_ConvertX86p16_8RGB332:

    ; check short
    cmp ecx,BYTE 16
    ja .L3


.L1: ; short loop
    mov al,[esi+0]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    and eax,BYTE 11000b         ; blue
    shr eax,3
    and ebx,11100000000b        ; green
    shr ebx,6
    and edx,1110000000000000b   ; red
    shr edx,8
    add eax,ebx
    add eax,edx
    mov [edi],al
    add esi,BYTE 2
    inc edi
    dec ecx
    jnz .L1
.L2:
    retn

.L3: mov eax,edi
    and eax,BYTE 11b
    jz .L4
    mov al,[esi+0]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    and eax,BYTE 11000b         ; blue
    shr eax,3
    and ebx,11100000000b        ; green
    shr ebx,6
    and edx,1110000000000000b   ; red
    shr edx,8
    add eax,ebx
    add eax,edx
    mov [edi],al
    add esi,BYTE 2
    inc edi
    dec ecx
    jmp SHORT .L3

.L4: ; save ebp
    push ebp

    ; save count
    push ecx

    ; unroll 4 times
    shr ecx,2

    ; prestep
    mov dl,[esi+0]
    mov bl,[esi+1]
    mov dh,[esi+2]
        
.L5:     shl edx,16
        mov bh,[esi+3]
        
        shl ebx,16
        mov dl,[esi+4]

        mov dh,[esi+6]
        mov bl,[esi+5]

        and edx,00011000000110000001100000011000b
        mov bh,[esi+7]

        ror edx,16+3
        mov eax,ebx                                     ; setup eax for reds

        and ebx,00000111000001110000011100000111b
        and eax,11100000111000001110000011100000b       ; reds

        ror ebx,16-2
        add esi,BYTE 8

        ror eax,16
        add edi,BYTE 4

        add eax,ebx
        mov bl,[esi+1]                                  ; greens

        add eax,edx
        mov dl,[esi+0]                                  ; blues

        mov [edi-4],eax
        mov dh,[esi+2]

        dec ecx
        jnz .L5                 
    
    ; check tail
    pop ecx
    and ecx,BYTE 11b
    jz .L7

.L6: ; tail
    mov al,[esi+0]
    mov ah,[esi+1]
    mov ebx,eax
    mov edx,eax
    and eax,BYTE 11000b         ; blue
    shr eax,3
    and ebx,11100000000b        ; green
    shr ebx,6
    and edx,1110000000000000b   ; red
    shr edx,8
    add eax,ebx
    add eax,edx
    mov [edi],al
    add esi,BYTE 2
    inc edi
    dec ecx
    jnz .L6

.L7: pop ebp
    retn

%ifidn __OUTPUT_FORMAT__,elf32
section .note.GNU-stack noalloc noexec nowrite progbits
%endif
