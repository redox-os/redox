;
; mmx format converter main loops for HERMES
; Some routines Copyright (c) 1998 Christian Nentwich (c.nentwich@cs.ucl.ac.uk)
; This source code is licensed under the GNU LGPL
; 
; Please refer to the file COPYING.LIB contained in the distribution for
; licensing conditions		
;

BITS 32

%include "common.inc"

SDL_FUNC _ConvertMMX

SECTION .text
		
;; _ConvertMMX:	 
;; [ESP+8] ConverterInfo*
;; --------------------------------------------------------------------------
;; ConverterInfo (ebp+..)
;;   0:	void *s_pixels
;;   4:	int s_width
;;   8:	int s_height
;;  12:	int s_add
;;  16:	void *d_pixels
;;  20:	int d_width
;;  24:	int d_height
;;  28:	int d_add
;;  32:	void (*converter_function)() 
;;  36: int32 *lookup
	
_ConvertMMX:
	push ebp
	mov ebp,esp

; Save the registers used by the blitters, necessary for optimized code
	pusha

	mov eax,[ebp+8]

        cmp dword [eax+4],BYTE 0
	je endconvert
	
	mov ebp,eax
	
	mov esi,[ebp+0]
	mov edi,[ebp+16]
	
y_loop:	
	mov ecx,[ebp+4]

	call [ebp+32]

	add esi,[ebp+12]
	add edi,[ebp+28]
	
	dec dword  [ebp+8]
	jnz y_loop

	
; Restore the registers used by the blitters, necessary for optimized code
	popa

	pop ebp

endconvert:
	emms
	
	ret		

%ifidn __OUTPUT_FORMAT__,elf32
section .note.GNU-stack noalloc noexec nowrite progbits
%endif
