    SECTION .data
msg:	db "Access Bad Data",10
len:	equ $-msg

	SECTION .text

        global _start
_start:
	mov	edx,len
	mov	ecx,msg
	mov	ebx,1
	mov	eax,4
	int	0x80

	mov ebx, [0]
	int3

	mov	eax, 1
	mov	ebx, 0
	int	0x80
