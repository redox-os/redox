;
; x86 format converters for HERMES
; Some routines Copyright (c) 1998 Christian Nentwich (brn@eleet.mcb.at)
; This source code is licensed under the GNU LGPL
; 
; Please refer to the file COPYING.LIB contained in the distribution for
; licensing conditions		
;
; Most routines are (c) Glenn Fiedler (ptc@gaffer.org), used with permission
; 

BITS 32

%include "common.inc"

SDL_FUNC _ConvertX86p32_32BGR888
SDL_FUNC _ConvertX86p32_32RGBA888
SDL_FUNC _ConvertX86p32_32BGRA888
SDL_FUNC _ConvertX86p32_24RGB888	
SDL_FUNC _ConvertX86p32_24BGR888
SDL_FUNC _ConvertX86p32_16RGB565
SDL_FUNC _ConvertX86p32_16BGR565
SDL_FUNC _ConvertX86p32_16RGB555
SDL_FUNC _ConvertX86p32_16BGR555
SDL_FUNC _ConvertX86p32_8RGB332

SECTION .text

;; _Convert_*
;; Paramters:	
;;   ESI = source 
;;   EDI = dest
;;   ECX = amount (NOT 0!!! (the _ConvertX86 routine checks for that though))
;; Destroys:
;;   EAX, EBX, EDX


_ConvertX86p32_32BGR888:

    ; check short
    cmp ecx,BYTE 32
    ja .L3

.L1: ; short loop
    mov edx,[esi]
    bswap edx
    ror edx,8
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; save ebp
    push ebp

    ; unroll four times
    mov ebp,ecx
    shr ebp,2
    
    ; save count
    push ecx

.L4:    mov eax,[esi]
        mov ebx,[esi+4]

        bswap eax

        bswap ebx

        ror eax,8
        mov ecx,[esi+8]

        ror ebx,8
        mov edx,[esi+12]

        bswap ecx

        bswap edx

        ror ecx,8
        mov [edi+0],eax

        ror edx,8
        mov [edi+4],ebx

        mov [edi+8],ecx
        mov [edi+12],edx

        add esi,BYTE 16
        add edi,BYTE 16

        dec ebp
        jnz .L4                 

    ; check tail
    pop ecx
    and ecx,BYTE 11b
    jz .L6

.L5: ; tail loop
    mov edx,[esi]
    bswap edx
    ror edx,8
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L5

.L6: pop ebp
    retn
	

	
		
_ConvertX86p32_32RGBA888:
	
    ; check short
    cmp ecx,BYTE 32
    ja .L3

.L1: ; short loop
    mov edx,[esi]
    rol edx,8
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; save ebp
    push ebp

    ; unroll four times
    mov ebp,ecx
    shr ebp,2
    
    ; save count
    push ecx

.L4:    mov eax,[esi]
        mov ebx,[esi+4]

        rol eax,8
        mov ecx,[esi+8]

        rol ebx,8
        mov edx,[esi+12]

        rol ecx,8
        mov [edi+0],eax

        rol edx,8
        mov [edi+4],ebx

        mov [edi+8],ecx
        mov [edi+12],edx

        add esi,BYTE 16
        add edi,BYTE 16

        dec ebp
        jnz .L4                 

    ; check tail
    pop ecx
    and ecx,BYTE 11b
    jz .L6

.L5: ; tail loop
    mov edx,[esi]
    rol edx,8
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L5

.L6: pop ebp
    retn

	


_ConvertX86p32_32BGRA888:

    ; check short
    cmp ecx,BYTE 32
    ja .L3

.L1: ; short loop
    mov edx,[esi]
    bswap edx
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L1
.L2:
    retn

.L3: ; save ebp
    push ebp

    ; unroll four times
    mov ebp,ecx
    shr ebp,2
    
    ; save count
    push ecx

.L4:    mov eax,[esi]
        mov ebx,[esi+4]

        mov ecx,[esi+8]
        mov edx,[esi+12]

        bswap eax

        bswap ebx

        bswap ecx

        bswap edx

        mov [edi+0],eax
        mov [edi+4],ebx

        mov [edi+8],ecx
        mov [edi+12],edx

        add esi,BYTE 16
        add edi,BYTE 16

        dec ebp
        jnz .L4                 

    ; check tail
    pop ecx
    and ecx,BYTE 11b
    jz .L6

.L5: ; tail loop
    mov edx,[esi]
    bswap edx
    mov [edi],edx
    add esi,BYTE 4
    add edi,BYTE 4
    dec ecx
    jnz .L5

.L6: pop ebp
    retn


	
	
;; 32 bit RGB 888 to 24 BIT RGB 888

_ConvertX86p32_24RGB888:

	; check short
	cmp ecx,BYTE 32
	ja .L3

.L1:	; short loop
	mov al,[esi]
	mov bl,[esi+1]
	mov dl,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jnz .L1
.L2:
	retn

.L3:	;	 head
	mov edx,edi
	and edx,BYTE 11b
	jz .L4
	mov al,[esi]
	mov bl,[esi+1]
	mov dl,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jmp SHORT .L3

.L4: ; unroll 4 times
	push ebp
	mov ebp,ecx
	shr ebp,2

    ; save count
	push ecx

.L5:    mov eax,[esi]                   ; first dword            eax = [A][R][G][B]
        mov ebx,[esi+4]                 ; second dword           ebx = [a][r][g][b]

        shl eax,8                       ;                        eax = [R][G][B][.]
        mov ecx,[esi+12]                ; third dword            ecx = [a][r][g][b]

        shl ebx,8                       ;                        ebx = [r][g][b][.]
        mov al,[esi+4]                  ;                        eax = [R][G][B][b]

        ror eax,8                       ;                        eax = [b][R][G][B] (done)
        mov bh,[esi+8+1]                ;                        ebx = [r][g][G][.]

        mov [edi],eax
        add edi,BYTE 3*4

        shl ecx,8                       ;                        ecx = [r][g][b][.]
        mov bl,[esi+8+0]                ;                        ebx = [r][g][G][B]

        rol ebx,16                      ;                        ebx = [G][B][r][g] (done)
        mov cl,[esi+8+2]                ;                        ecx = [r][g][b][R] (done)

        mov [edi+4-3*4],ebx
        add esi,BYTE 4*4
        
        mov [edi+8-3*4],ecx
        dec ebp

        jnz .L5

    ; check tail
	pop ecx
	and ecx,BYTE 11b
	jz .L7

.L6: ; tail loop
	mov al,[esi]
	mov bl,[esi+1]
	mov dl,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jnz .L6

.L7:	pop ebp
	retn




;; 32 bit RGB 888 to 24 bit BGR 888

_ConvertX86p32_24BGR888:

	; check short
	cmp ecx,BYTE 32
	ja .L3

.L1:	; short loop
	mov dl,[esi]
	mov bl,[esi+1]
	mov al,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jnz .L1
.L2:
	retn

.L3: ; head
	mov edx,edi
	and edx,BYTE 11b
	jz .L4
	mov dl,[esi]
	mov bl,[esi+1]
	mov al,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jmp SHORT .L3

.L4:	; unroll 4 times
	push ebp
	mov ebp,ecx
	shr ebp,2

	; save count
	push ecx

.L5:
	mov eax,[esi]                   ; first dword            eax = [A][R][G][B]
        mov ebx,[esi+4]                 ; second dword           ebx = [a][r][g][b]

        bswap eax                       ;                        eax = [B][G][R][A]

        bswap ebx                       ;                        ebx = [b][g][r][a]

        mov al,[esi+4+2]                ;                        eax = [B][G][R][r] 
        mov bh,[esi+4+4+1]              ;                        ebx = [b][g][G][a]

        ror eax,8                       ;                        eax = [r][B][G][R] (done)
        mov bl,[esi+4+4+2]              ;                        ebx = [b][g][G][R]

        ror ebx,16                      ;                        ebx = [G][R][b][g] (done)
        mov [edi],eax
    
        mov [edi+4],ebx
        mov ecx,[esi+12]                ; third dword            ecx = [a][r][g][b]
        
        bswap ecx                       ;                        ecx = [b][g][r][a]
        
        mov cl,[esi+8]                  ;                        ecx = [b][g][r][B] (done)
        add esi,BYTE 4*4

        mov [edi+8],ecx
        add edi,BYTE 3*4

        dec ebp
        jnz .L5

	; check tail
	pop ecx
	and ecx,BYTE 11b
	jz .L7

.L6:	; tail loop
	mov dl,[esi]
	mov bl,[esi+1]
	mov al,[esi+2]
	mov [edi],al
	mov [edi+1],bl
	mov [edi+2],dl
	add esi,BYTE 4
	add edi,BYTE 3
	dec ecx
	jnz .L6

.L7:
	pop ebp
	retn
 

	
		
;; 32 bit RGB 888 to 16 BIT RGB 565 

_ConvertX86p32_16RGB565:
	; check short
	cmp ecx,BYTE 16
	ja .L3

.L1: ; short loop
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
        and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx
	jnz .L1

.L2:				; End of short loop
	retn

	
.L3:	; head
	mov ebx,edi
	and ebx,BYTE 11b
	jz .L4
	
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
	and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx

.L4:	 
    ; save count
	push ecx

    ; unroll twice
	shr ecx,1
    
    ; point arrays to end
	lea esi,[esi+ecx*8]
	lea edi,[edi+ecx*4]

    ; negative counter 
	neg ecx
	jmp SHORT .L6

.L5:	    
	mov [edi+ecx*4-4],eax
.L6:	
	mov eax,[esi+ecx*8]

        shr ah,2
        mov ebx,[esi+ecx*8+4]

        shr eax,3
        mov edx,[esi+ecx*8+4]

        shr bh,2
        mov dl,[esi+ecx*8+2]

        shl ebx,13
        and eax,000007FFh
        
        shl edx,8
        and ebx,07FF0000h

        and edx,0F800F800h
        add eax,ebx

        add eax,edx
        inc ecx

        jnz .L5                 

	mov [edi+ecx*4-4],eax

    ; tail
	pop ecx
	test cl,1
	jz .L7
	
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
	and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2

.L7:	
	retn



	
;; 32 bit RGB 888 to 16 BIT BGR 565 

_ConvertX86p32_16BGR565:
	
	; check short
	cmp ecx,BYTE 16
	ja .L3

.L1:	; short loop
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx
	jnz .L1
.L2:
	retn

.L3:	; head
	mov ebx,edi
	and ebx,BYTE 11b
	jz .L4   
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx

.L4:	; save count
	push ecx

	; unroll twice
	shr ecx,1
    
	; point arrays to end
	lea esi,[esi+ecx*8]
	lea edi,[edi+ecx*4]

	; negative count
	neg ecx
	jmp SHORT .L6

.L5:
	mov [edi+ecx*4-4],eax            
.L6:
	mov edx,[esi+ecx*8+4]

        mov bh,[esi+ecx*8+4]                       
        mov ah,[esi+ecx*8]                       

        shr bh,3
        mov al,[esi+ecx*8+1]             

        shr ah,3
        mov bl,[esi+ecx*8+5]           

        shl eax,3
        mov dl,[esi+ecx*8+2]

        shl ebx,19
        and eax,0000FFE0h              
                
        shr edx,3
        and ebx,0FFE00000h             
        
        and edx,001F001Fh               
        add eax,ebx

        add eax,edx
        inc ecx

        jnz .L5                 

	mov [edi+ecx*4-4],eax            

	; tail
	pop ecx
	and ecx,BYTE 1
	jz .L7
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111100b
	shl eax,3
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2

.L7:
	retn


	
	
;; 32 BIT RGB TO 16 BIT RGB 555

_ConvertX86p32_16RGB555:

	; check short
	cmp ecx,BYTE 16
	ja .L3

.L1:	; short loop
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx
	jnz .L1
.L2:
	retn

.L3:	; head
	mov ebx,edi
        and ebx,BYTE 11b
	jz .L4   
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx

.L4:	; save count
	push ecx

	; unroll twice
	shr ecx,1
    
	; point arrays to end
	lea esi,[esi+ecx*8]
	lea edi,[edi+ecx*4]

	; negative counter 
	neg ecx
	jmp SHORT .L6

.L5:
	mov [edi+ecx*4-4],eax
.L6:
	mov eax,[esi+ecx*8]

        shr ah,3
        mov ebx,[esi+ecx*8+4]

        shr eax,3
        mov edx,[esi+ecx*8+4]

        shr bh,3
        mov dl,[esi+ecx*8+2]

        shl ebx,13
        and eax,000007FFh
        
        shl edx,7
        and ebx,07FF0000h

        and edx,07C007C00h
        add eax,ebx

        add eax,edx
        inc ecx

        jnz .L5                 

	mov [edi+ecx*4-4],eax

	; tail
	pop ecx
	and ecx,BYTE 1
	jz .L7
	mov bl,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov ah,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2

.L7:
	retn




;; 32 BIT RGB TO 16 BIT BGR 555
	
_ConvertX86p32_16BGR555:
	
	; check short
	cmp ecx,BYTE 16
	ja .L3


.L1:	; short loop
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx
	jnz .L1
.L2:
	retn

.L3:	; head
	mov ebx,edi
        and ebx,BYTE 11b
	jz .L4   
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2
	dec ecx

.L4:	; save count
	push ecx

	; unroll twice
	shr ecx,1
    
	; point arrays to end
	lea esi,[esi+ecx*8]
	lea edi,[edi+ecx*4]

	; negative counter 
	neg ecx
	jmp SHORT .L6

.L5:
	mov [edi+ecx*4-4],eax            
.L6:
	mov edx,[esi+ecx*8+4]

        mov bh,[esi+ecx*8+4]                       
        mov ah,[esi+ecx*8]                       

        shr bh,3
        mov al,[esi+ecx*8+1]             

        shr ah,3
        mov bl,[esi+ecx*8+5]           

        shl eax,2
        mov dl,[esi+ecx*8+2]

        shl ebx,18
        and eax,00007FE0h              
                
        shr edx,3
        and ebx,07FE00000h             
        
        and edx,001F001Fh               
        add eax,ebx

        add eax,edx
        inc ecx

        jnz .L5                 

	mov [edi+ecx*4-4],eax            

	; tail
	pop ecx
	and ecx,BYTE 1
	jz .L7
	mov ah,[esi+0]    ; blue
	mov al,[esi+1]    ; green
	mov bl,[esi+2]    ; red
	shr ah,3
	and al,11111000b
	shl eax,2
	shr bl,3
	add al,bl
	mov [edi+0],al
	mov [edi+1],ah
	add esi,BYTE 4
	add edi,BYTE 2

.L7:
	retn




	
;; FROM 32 BIT RGB to 8 BIT RGB (rrrgggbbb)
;; This routine writes FOUR pixels at once (dword) and then, if they exist
;; the trailing three pixels
_ConvertX86p32_8RGB332:

	
.L_ALIGNED:
	push ecx

	shr ecx,2		; We will draw 4 pixels at once
	jnz .L1
	
	jmp .L2			; short jump out of range :(
	
.L1:
	mov eax,[esi]		; first pair of pixels
	mov edx,[esi+4]

	shr dl,6
	mov ebx,eax

	shr al,6
	and ah,0e0h

	shr ebx,16
	and dh,0e0h
	
	shr ah,3
	and bl,0e0h

	shr dh,3
	
	or al,bl
	
	mov ebx,edx	
	or al,ah
	
	shr ebx,16
	or dl,dh

	and bl,0e0h
	
	or dl,bl

	mov ah,dl

	
		
	mov ebx,[esi+8]		; second pair of pixels

	mov edx,ebx
	and bh,0e0h

	shr bl,6
	and edx,0e00000h

	shr edx,16

	shr bh,3

	ror eax,16
	or bl,dl

	mov edx,[esi+12]
	or bl,bh
	
	mov al,bl

	mov ebx,edx
	and dh,0e0h

	shr dl,6
	and ebx,0e00000h
	
	shr dh,3
	mov ah,dl

	shr ebx,16
	or ah,dh

	or ah,bl

	rol eax,16
	add esi,BYTE 16
			
	mov [edi],eax	
	add edi,BYTE 4
	
	dec ecx
	jz .L2			; L1 out of range for short jump :(
	
	jmp .L1
.L2:
	
	pop ecx
	and ecx,BYTE 3		; mask out number of pixels to draw
	
	jz .L4			; Nothing to do anymore

.L3:
	mov eax,[esi]		; single pixel conversion for trailing pixels

        mov ebx,eax

        shr al,6
        and ah,0e0h

        shr ebx,16

        shr ah,3
        and bl,0e0h

        or al,ah
        or al,bl

        mov [edi],al

        inc edi
        add esi,BYTE 4

	dec ecx
	jnz .L3
	
.L4:	
	retn

%ifidn __OUTPUT_FORMAT__,elf32
section .note.GNU-stack noalloc noexec nowrite progbits
%endif
