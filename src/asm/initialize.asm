SECTION .text
[BITS 16]
initialize:
.fpu: ;enable fpu
	mov eax, cr4
	or eax, 0x200
	mov cr4, eax
	mov eax, 0xB7F
	push eax
	fldcw [esp]
	pop eax
	ret

.sse: ;enable sse
	mov eax, cr0
	and al, 11111011b
	or al, 00000010b
	mov cr0, eax
	mov eax, cr4
	or ax, 0000011000000000b
	mov cr4, eax
	ret

.pic:	;sets up IRQs at int 20-2F
	mov al, 0x11
	out 0x20, al
	out 0xA0, al
	mov al, 0x20	;IRQ0 vector
	out 0x21, al
	mov al, 0x28	;IRQ8 vector
	out 0xA1, al
	mov al, 4
	out 0x21, al
	mov al, 2
	out 0xA1, al
	mov al, 1
	out 0x21, al
	out 0xA1, al
	xor al, al		;no IRQ masks
	out 0x21, al
	out 0xA1, al
	mov al, 0x20	;reset PIC's
	out 0xA0, al
	out 0x20, al
	ret
