ORG 0x7C00
SECTION .text
USE16

boot: ; dl comes with disk
    ; initialize segment registers
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax

    ; initialize stack
    mov sp, 0x7C00

    ; initialize CS
    push ax
    push word .set_cs
    retf

.set_cs:

    ; save disk number
    mov [disk], dl

    mov si, name
    call print
    call print_line

    mov bh, 0
    mov bl, [disk]

    mov eax, (startup_start - boot) / 512
    mov bx, startup_start
    mov cx, (startup_end - startup_start) / 512
    xor dx, dx
    call load

    mov si, finished
    call print
    call print_line

    jmp startup

; load some sectors from disk to a buffer in memory
; buffer has to be below 1MiB
; IN
;   ax: start sector
;   bx: offset of buffer
;   cx: number of sectors (512 Bytes each)
;   dx: segment of buffer
; CLOBBER
;   ax, bx, cx, dx, si
; TODO rewrite to (eventually) move larger parts at once
; if that is done increase buffer_size_sectors in startup-common to that (max 0x80000 - startup_end)
load:
    cmp cx, 127
    jbe .good_size

    pusha
    mov cx, 127
    call load
    popa
    add ax, 127
    add dx, 127 * 512 / 16
    sub cx, 127

    jmp load
.good_size:
    mov [DAPACK.addr], eax
    mov [DAPACK.buf], bx
    mov [DAPACK.count], cx
    mov [DAPACK.seg], dx

    call print_dapack

    mov dl, [disk]
    mov si, DAPACK
    mov ah, 0x42
    int 0x13
    jc error
    ret

  ; store some sectors to disk from a buffer in memory
  ; buffer has to be below 1MiB
  ; IN
  ;   ax: start sector
  ;   bx: offset of buffer
  ;   cx: number of sectors (512 Bytes each)
  ;   dx: segment of buffer
  ; CLOBBER
  ;   ax, bx, cx, dx, si
  ; TODO rewrite to (eventually) move larger parts at once
  ; if that is done increase buffer_size_sectors in startup-common to that (max 0x80000 - startup_end)
  store:
      cmp cx, 127
      jbe .good_size

      pusha
      mov cx, 127
      call store
      popa
      add ax, 127
      add dx, 127 * 512 / 16
      sub cx, 127

      jmp store
  .good_size:
      mov [DAPACK.addr], eax
      mov [DAPACK.buf], bx
      mov [DAPACK.count], cx
      mov [DAPACK.seg], dx

      call print_dapack

      mov dl, [disk]
      mov si, DAPACK
      mov ah, 0x43
      int 0x13
      jc error
      ret

print_dapack:
    mov bx, [DAPACK.addr + 2]
    call print_num

    mov bx, [DAPACK.addr]
    call print_num

    mov al, '#'
    call print_char

    mov bx, [DAPACK.count]
    call print_num

    mov al, ' '
    call print_char

    mov bx, [DAPACK.seg]
    call print_num

    mov al, ':'
    call print_char

    mov bx, [DAPACK.buf]
    call print_num

    jmp print_line

error:
    mov bh, 0
    mov bl, ah
    call print_num

    mov al, ' '
    call print_char

    mov si, errored
    call print
    call print_line
.halt:
    cli
    hlt
    jmp .halt

%include "print16.asm"

name: db "Redox Loader - Stage One",0
errored: db "Could not read disk",0
finished: db "Redox Loader - Stage Two",0

disk: db 0

DAPACK:
        db 0x10
        db 0
.count: dw 0 ; int 13 resets this to # of blocks actually read/written
.buf:   dw 0 ; memory buffer destination address (0:7c00)
.seg:   dw 0 ; in memory page zero
.addr:  dq 0 ; put the lba to read in this spot

times 510-($-$$) db 0
db 0x55
db 0xaa
