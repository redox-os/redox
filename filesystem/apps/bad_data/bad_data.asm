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

	mov eax, 0xAAAAAAAA
    mov ebx, 0xBBBBBBBB
    mov ecx, 0xCCCCCCCC
    mov edx, 0xDDDDDDDD
    mov esi, 0x51515151
    mov edi, 0xD1D1D1D1
	mov ebx, [0]

	mov	eax, 1
	mov	ebx, 0
	int	0x80
