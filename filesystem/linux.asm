	SECTION .data
msg:	db "Linux ABI STDIO",10
len:	equ $-msg

	SECTION .text

        global _start
_start:
	mov	edx,len
	mov	ecx,msg
	mov	ebx,1
	mov	eax,4

	int	0x80
	mov	ebx,0
	mov	eax,1
	int	0x80
