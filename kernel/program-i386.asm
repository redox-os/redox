extern _start
section .text
global __libc_csu_init
__libc_csu_init:
	ret
global __libc_csu_fini
__libc_csu_fini:
	ret
global __libc_start_main
__libc_start_main:
	call _start
	mov ebx, eax
	mov eax, 1
	int 0x80
