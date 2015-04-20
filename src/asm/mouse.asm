mouse:
SECTION .text
[BITS 64]
.init:
	call .wait1
	mov al, 0xA8
	out 0x64, al
	
	call .wait1
	mov al, 0x20
	out 0x64, al
	call .wait0
	in al, 0x60
	mov ah, al
	or ah, 2
	call .wait1
	mov al, 0x60
	out 0x64, al
	call .wait1
	mov al, ah
	out 0x60, al
	
	mov bl, 0xF6
	call .write
	call .read
	
	mov bl, 0xF4
	call .write
	call .read
	ret
	
.wait0:
	mov rcx, 100000
.wait0lp:
	in al, 0x64
	test al, 1
	loopz .wait0lp
.mousebyte:
	ret
	
.wait1:
	mov rcx, 100000
.wait1lp:
	in al, 0x64
	test al, 2
	loopnz .wait1lp
	ret
	
.write:
	call .wait1
	mov al, 0xD4
	out 0x64, al
	call .wait1
	mov al, bl
	out 0x60, al
	ret
	
.read:
	call .wait0
	in al, 0x60
	ret
	