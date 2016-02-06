SECTION .text
USE16
; provide function for printing in x86 real mode


; a newline
newline: db 0xD, 0xA, 0

; print a string and a newline
; IN
;   si: points at zero-terminated String
; CLOBBER
;   ax
print_line:
    mov si, newline
    call print
    ret

; print a string
; IN
;   si: points at zero-terminated String
; CLOBBER
;   ax
print:
    lodsb
    test al, al
    jz .done
    call print_char
    jmp print
.done:
    ret

; print a character
; IN
;   al: character to print
; CLOBBER
;   ah
print_char:
    mov ah, 0x0e
    int 0x10
    ret

; print a number in hex
; IN
;   bx: the number
; CLOBBER
;   cx, ax
print_num:
    mov cx, 4
.lp:
    mov al, bh
    shr al, 4

    cmp al, 0xA
    jb .below_0xA

    add al, 'A' - 0xA - '0'
.below_0xA:
    add al, '0'

    call print_char

    shl bx, 4
    loop .lp

    ret
