;
; pII-optimised MMX format converters for HERMES
; Copyright (c) 1998 Christian Nentwich (c.nentwich@cs.ucl.ac.uk)
;   and (c) 1999 Jonathan Matthew (jmatthew@uq.net.au)
; This source code is licensed under the GNU LGPL
; 
; Please refer to the file COPYING.LIB contained in the distribution for
; licensing conditions		
;
; COPYRIGHT NOTICE
; 
; This file partly contains code that is (c) Intel Corporation, specifically
; the mode detection routine, and the converter to 15 bit (8 pixel
; conversion routine from the mmx programming tutorial pages).
;
;
; These routines aren't exactly pII optimised - it's just that as they
; are, they're terrible on p5 MMXs, but less so on pIIs.  Someone needs to
; optimise them for p5 MMXs..

BITS 32

%include "common.inc"
	
SDL_FUNC _ConvertMMXpII32_24RGB888
SDL_FUNC _ConvertMMXpII32_16RGB565
SDL_FUNC _ConvertMMXpII32_16BGR565
SDL_FUNC _ConvertMMXpII32_16RGB555
SDL_FUNC _ConvertMMXpII32_16BGR555

;; Macros for conversion routines

%macro _push_immq_mask 1
	push dword %1
	push dword %1
%endmacro

%macro load_immq 2
	_push_immq_mask %2
	movq %1, [esp]
%endmacro

%macro pand_immq 2
	_push_immq_mask %2
	pand %1, [esp]
%endmacro

%define CLEANUP_IMMQ_LOADS(num) \
	add esp, byte 8 * num

%define mmx32_rgb888_mask 00ffffffh
%define mmx32_rgb565_b 000000f8h
%define mmx32_rgb565_g 0000fc00h
%define mmx32_rgb565_r 00f80000h

%define mmx32_rgb555_rb 00f800f8h
%define mmx32_rgb555_g 0000f800h
%define mmx32_rgb555_mul 20000008h
%define mmx32_bgr555_mul 00082000h

SECTION .text

_ConvertMMXpII32_24RGB888:

        ; set up mm6 as the mask, mm7 as zero
        load_immq mm6, mmx32_rgb888_mask
        CLEANUP_IMMQ_LOADS(1)
        pxor mm7, mm7

        mov edx, ecx                    ; save ecx
        and ecx, 0fffffffch             ; clear lower two bits
        jnz .L1
        jmp .L2

.L1:

        movq mm0, [esi]                 ; A R G B a r g b
        pand mm0, mm6                   ; 0 R G B 0 r g b
        movq mm1, [esi+8]               ; A R G B a r g b
        pand mm1, mm6                   ; 0 R G B 0 r g b

        movq mm2, mm0                   ; 0 R G B 0 r g b
        punpckhdq mm2, mm7              ; 0 0 0 0 0 R G B
        punpckldq mm0, mm7              ; 0 0 0 0 0 r g b
        psllq mm2, 24                   ; 0 0 R G B 0 0 0
        por mm0, mm2                    ; 0 0 R G B r g b

        movq mm3, mm1                   ; 0 R G B 0 r g b
        psllq mm3, 48                   ; g b 0 0 0 0 0 0
        por mm0, mm3                    ; g b R G B r g b

        movq mm4, mm1                   ; 0 R G B 0 r g b
        punpckhdq mm4, mm7              ; 0 0 0 0 0 R G B
        punpckldq mm1, mm7              ; 0 0 0 0 0 r g b
        psrlq mm1, 16                   ; 0 0 0 R G B 0 r
        psllq mm4, 8                    ; 0 0 0 0 R G B 0
        por mm1, mm4                    ; 0 0 0 0 R G B r

        movq [edi], mm0
        add esi, BYTE 16
        movd [edi+8], mm1
        add edi, BYTE 12
        sub ecx, BYTE 4
        jnz .L1

.L2:
        mov ecx, edx
        and ecx, BYTE 3
        jz .L4
.L3:
        mov al, [esi]
        mov bl, [esi+1]
        mov dl, [esi+2]
        mov [edi], al
        mov [edi+1], bl
        mov [edi+2], dl
        add esi, BYTE 4
        add edi, BYTE 3
        dec ecx
        jnz .L3
.L4:
        retn



_ConvertMMXpII32_16RGB565:

        ; set up masks
        load_immq mm5, mmx32_rgb565_b
        load_immq mm6, mmx32_rgb565_g
        load_immq mm7, mmx32_rgb565_r
        CLEANUP_IMMQ_LOADS(3)

        mov edx, ecx
        shr ecx, 2
        jnz .L1
        jmp .L2         ; not necessary at the moment, but doesn't hurt (much)

.L1:
        movq mm0, [esi]         ; argb
        movq mm1, mm0           ; argb
        pand mm0, mm6           ; 00g0
        movq mm3, mm1           ; argb
        pand mm1, mm5           ; 000b
        pand mm3, mm7           ; 0r00
        pslld mm1, 2            ; 0 0 000000bb bbb00000
        por mm0, mm1            ; 0 0 ggggggbb bbb00000
        psrld mm0, 5            ; 0 0 00000ggg gggbbbbb

        movq mm4, [esi+8]       ; argb
        movq mm2, mm4           ; argb
        pand mm4, mm6           ; 00g0
        movq mm1, mm2           ; argb
        pand mm2, mm5           ; 000b
        pand mm1, mm7           ; 0r00
        pslld mm2, 2            ; 0 0 000000bb bbb00000
        por mm4, mm2            ; 0 0 ggggggbb bbb00000
        psrld mm4, 5            ; 0 0 00000ggg gggbbbbb

        packuswb mm3, mm1       ; R 0 r 0
        packssdw mm0, mm4       ; as above.. ish
        por mm0, mm3            ; done.
        movq [edi], mm0

        add esi, 16
        add edi, 8
        dec ecx
        jnz .L1

.L2:
        mov ecx, edx
        and ecx, BYTE 3
        jz .L4
.L3:
        mov al, [esi]
        mov bh, [esi+1]
        mov ah, [esi+2]
        shr al, 3
        and eax, 0F81Fh            ; BYTE?
        shr ebx, 5
        and ebx, 07E0h             ; BYTE?
        add eax, ebx
        mov [edi], al
        mov [edi+1], ah
        add esi, BYTE 4
        add edi, BYTE 2
        dec ecx
        jnz .L3

.L4:
	retn

	
_ConvertMMXpII32_16BGR565:

        load_immq mm5, mmx32_rgb565_r
        load_immq mm6, mmx32_rgb565_g
        load_immq mm7, mmx32_rgb565_b
        CLEANUP_IMMQ_LOADS(3)

        mov edx, ecx
        shr ecx, 2
        jnz .L1
        jmp .L2

.L1:
        movq mm0, [esi]                 ; a r g b
        movq mm1, mm0                   ; a r g b
        pand mm0, mm6                   ; 0 0 g 0
        movq mm3, mm1                   ; a r g b
        pand mm1, mm5                   ; 0 r 0 0
        pand mm3, mm7                   ; 0 0 0 b

        psllq mm3, 16                   ; 0 b 0 0
        psrld mm1, 14                   ; 0 0 000000rr rrr00000
        por mm0, mm1                    ; 0 0 ggggggrr rrr00000
        psrld mm0, 5                    ; 0 0 00000ggg gggrrrrr

        movq mm4, [esi+8]               ; a r g b
        movq mm2, mm4                   ; a r g b
        pand mm4, mm6                   ; 0 0 g 0
        movq mm1, mm2                   ; a r g b
        pand mm2, mm5                   ; 0 r 0 0
        pand mm1, mm7                   ; 0 0 0 b

        psllq mm1, 16                   ; 0 b 0 0
        psrld mm2, 14                   ; 0 0 000000rr rrr00000
        por mm4, mm2                    ; 0 0 ggggggrr rrr00000
        psrld mm4, 5                    ; 0 0 00000ggg gggrrrrr

        packuswb mm3, mm1               ; BBBBB000 00000000 bbbbb000 00000000
        packssdw mm0, mm4               ; 00000GGG GGGRRRRR 00000GGG GGGRRRRR
        por mm0, mm3                    ; BBBBBGGG GGGRRRRR bbbbbggg gggrrrrr
        movq [edi], mm0

        add esi, BYTE 16
        add edi, BYTE 8
        dec ecx
        jnz .L1

.L2:
        and edx, BYTE 3
        jz .L4
.L3:
        mov al, [esi+2]
        mov bh, [esi+1]
        mov ah, [esi]
        shr al, 3
        and eax, 0F81Fh                    ; BYTE ?
        shr ebx, 5
        and ebx, 07E0h                     ; BYTE ?
        add eax, ebx
        mov [edi], al
        mov [edi+1], ah
        add esi, BYTE 4
        add edi, BYTE 2
        dec edx
        jnz .L3

.L4:
        retn

_ConvertMMXpII32_16BGR555:

        ; the 16BGR555 converter is identical to the RGB555 one,
        ; except it uses a different multiplier for the pmaddwd
        ; instruction.  cool huh.

        load_immq mm7, mmx32_bgr555_mul
        jmp _convert_bgr555_cheat

; This is the same as the Intel version.. they obviously went to
; much more trouble to expand/coil the loop than I did, so theirs
; would almost certainly be faster, even if only a little.
; I did rename 'mmx32_rgb555_add' to 'mmx32_rgb555_mul', which is
; (I think) a more accurate name..
_ConvertMMXpII32_16RGB555:

	load_immq mm7, mmx32_rgb555_mul
_convert_bgr555_cheat:
	load_immq mm6, mmx32_rgb555_g
	CLEANUP_IMMQ_LOADS(2)
        
	mov edx,ecx		           ; Save ecx 

        and ecx,DWORD 0fffffff8h            ; clear lower three bits
	jnz .L_OK
        jmp near .L2 

.L_OK:
	
	movq mm2,[esi+8]

	movq mm0,[esi]
	movq mm3,mm2

	pand_immq mm3, mmx32_rgb555_rb
	movq mm1,mm0

	pand_immq mm1, mmx32_rgb555_rb
	pmaddwd mm3,mm7

	CLEANUP_IMMQ_LOADS(2)

	pmaddwd mm1,mm7
	pand mm2,mm6

.L1:
	movq mm4,[esi+24]
	pand mm0,mm6

	movq mm5,[esi+16]
	por mm3,mm2

	psrld mm3,6
	por mm1,mm0

	movq mm0,mm4
	psrld mm1,6

	pand_immq mm0, mmx32_rgb555_rb
	packssdw mm1,mm3

	movq mm3,mm5
	pmaddwd mm0,mm7

	pand_immq mm3, mmx32_rgb555_rb
	pand mm4,mm6

	movq [edi],mm1			
	pmaddwd mm3,mm7

        add esi,BYTE 32
	por mm4,mm0

	pand mm5,mm6
	psrld mm4,6

	movq mm2,[esi+8]
	por mm5,mm3

	movq mm0,[esi]
	psrld mm5,6

	movq mm3,mm2
	movq mm1,mm0

	pand_immq mm3, mmx32_rgb555_rb
	packssdw mm5,mm4

	pand_immq mm1, mmx32_rgb555_rb
	pand mm2,mm6

	CLEANUP_IMMQ_LOADS(4)

	movq [edi+8],mm5
	pmaddwd mm3,mm7

	pmaddwd mm1,mm7
        add edi,BYTE 16
	
        sub ecx,BYTE 8
	jz .L2
        jmp .L1


.L2:	
	mov ecx,edx
	
        and ecx,BYTE 7
	jz .L4
	
.L3:	
	mov ebx,[esi]
        add esi,BYTE 4
	
        mov eax,ebx
        mov edx,ebx

        shr eax,3
        shr edx,6

        and eax,BYTE 0000000000011111b
        and edx,     0000001111100000b

        shr ebx,9

        or eax,edx

        and ebx,     0111110000000000b

        or eax,ebx

        mov [edi],ax
        add edi,BYTE 2

	dec ecx
	jnz .L3	

.L4:		
	retn

%ifidn __OUTPUT_FORMAT__,elf32
section .note.GNU-stack noalloc noexec nowrite progbits
%endif
