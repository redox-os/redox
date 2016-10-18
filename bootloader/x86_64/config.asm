SECTION .text
USE16

align 512, db 0

config:
  .xres: dw 0
  .yres: dw 0

times 512 - ($ - config) db 0

save_config:
    mov eax, (config - boot) / 512
    mov bx, config
    mov cx, 1
    xor dx, dx
    call store
    ret
