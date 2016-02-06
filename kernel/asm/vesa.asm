%include "asm/vesa.inc"
[section .text]
[BITS 16]
vesa:
.getcardinfo:
    mov ax, 0x4F00
    mov di, VBECardInfo
    int 0x10
    cmp ax, 0x4F
    je .edid
    mov eax, 1
    ret
.edid:
    cmp dword [.required], 0    ;if both required x and required y are set, forget this
    jne near .findmode
    mov ax, 0x4F15
    mov bx, 1
    xor cx, cx
    xor dx, dx
    mov di, VBEEDID
    int 0x10
    cmp ax, 0x4F
    jne near .noedid
    xor di, di
.lp:
    xor cx, cx
    mov cl, [di+VBEEDID.standardtiming]
    shl cx, 3
    add cx, 248
    push ecx
    call decshowrm
    mov al, 'x'
    call charrm
    pop ecx
    mov bx, cx
    inc di
    mov al, [di+VBEEDID.standardtiming]
    and al, 11000000b
    cmp al, VBEEDID.aspect.4.3
    jne .not43
    mov ax, 3
    mul cx
    mov cx, ax
    shr cx, 2
    jmp .gotres
.not43:
    cmp al, VBEEDID.aspect.5.4
    jne .not54
    shl cx, 2
    mov ax, cx
    mov cx, 5
    xor dx, dx
    div cx
    mov cx, ax
    jmp .gotres
.not54:
    cmp al, VBEEDID.aspect.16.10
    jne .not1610
    mov ax, 10
    mul cx
    mov cx, ax
    shr cx, 4
    jmp .gotres
.not1610:
    mov ax, 9
    mul cx
    mov cx, ax
    shr cx, 4
.gotres:
    call decshowrm
    mov si, .edidmsg
    call printrm
    inc di
    cmp di, 8
    jb .lp
    jmp .findmode
.noedid:
    mov si, .noedidmsg
    call printrm
    jmp .findmode
 .resetlist:
    ;if needed, reset mins/maxes/stuff
    xor cx, cx
    mov [.minx], cx
    mov [.miny], cx
    mov [.requiredx], cx
    mov [.requiredy], cx
    mov [.requiredmode], cx
.findmode:
    mov    si, [VBECardInfo.videomodeptr]
    mov ax, [VBECardInfo.videomodeptr+2]
    mov fs, ax
    sub si, 2
    mov cx, [.requiredmode]
    test cx, cx
    jnz .getmodeinfo
.searchmodes:
    add si, 2
    mov cx, [fs:si]
    cmp cx, 0xFFFF
    jne .getmodeinfo
    cmp word [.goodmode], 0
    je .resetlist
    jmp .findmode
.getmodeinfo:
    push esi
    mov [.currentmode], cx
    mov ax, 0x4F01
    mov di, VBEModeInfo
    int 0x10
    pop esi
    cmp ax, 0x4F
    je .foundmode
    mov eax, 1
    ret
.foundmode:
    ;check minimum values, really not minimums from an OS perspective but ugly for users
    cmp byte [VBEModeInfo.bitsperpixel], 32
    jb .searchmodes
.testx:
    mov cx, [VBEModeInfo.xresolution]
    cmp word [.requiredx], 0
    je .notrequiredx
    cmp cx, [.requiredx]
    je .testy
    jmp .searchmodes
.notrequiredx:
    cmp cx, [.minx]
    jb .searchmodes
.testy:
    mov cx, [VBEModeInfo.yresolution]
    cmp word [.requiredy], 0
    je .notrequiredy
    cmp cx, [.requiredy]
    jne .searchmodes    ;as if there weren't enough warnings, USE WITH CAUTION
    cmp word [.requiredx], 0
    jnz .setmode
    jmp .testgood
.notrequiredy:
    cmp cx, [.miny]
    jb .searchmodes
.testgood:
    mov cx, [.currentmode]
    mov [.goodmode], cx
    push esi
    call decshowrm
    mov al, ':'
    call charrm
    mov cx, [VBEModeInfo.xresolution]
    call decshowrm
    mov al, 'x'
    call charrm
    mov cx, [VBEModeInfo.yresolution]
    call decshowrm
    mov al, '@'
    call charrm
    xor ch, ch
    mov cl, [VBEModeInfo.bitsperpixel]
    call decshowrm
    mov si, .modeok
    call printrm
    xor ax, ax
    int 0x16
    pop esi
    cmp al, 'y'
    jne .searchmodes
.setmode:
    mov bx, [.currentmode]
    cmp bx, 0
    je .nomode
    or bx, 0x4000
    mov ax, 0x4F02
    int 0x10
.nomode:
    cmp ax, 0x4F
    je .returngood
    mov eax, 1
    ret
.returngood:
    xor eax, eax
    ret

.minx dw 640
.miny dw 480
.required:
.requiredx dw 800    ;USE THESE WITH CAUTION
.requiredy dw 600
.requiredmode dw 0

.noedidmsg db "EDID not supported.",10,13,0
.edidmsg    db " is supported.",10,13,0
.modeok db 10,13,"Is this OK?(y/n)",10,13,0

.goodmode dw 0
.currentmode dw 0
;useful functions

decshowrm:
    mov si, .number
.clear:
    mov al, "0"
    mov [si], al
    inc si
    cmp si, .numberend
    jb .clear
    dec si
    call convertrm
    mov si, .number
.lp:
    lodsb
    cmp si, .numberend
    jae .end
    cmp al, "0"
    jbe .lp
.end:
    dec si
    call printrm
    ret

.number times 7 db 0
.numberend db 0

convertrm:
    dec si
    mov bx, si        ;place to convert into must be in si, number to convert must be in cx
.cnvrt:
    mov si, bx
    sub si, 4
.ten4:    inc si
    cmp cx, 10000
    jb .ten3
    sub cx, 10000
    inc byte [si]
    jmp .cnvrt
.ten3:    inc si
    cmp cx, 1000
    jb .ten2
    sub cx, 1000
    inc byte [si]
    jmp .cnvrt
.ten2:    inc si
    cmp cx, 100
    jb .ten1
    sub cx, 100
    inc byte [si]
    jmp .cnvrt
.ten1:    inc si
    cmp cx, 10
    jb .ten0
    sub cx, 10
    inc byte [si]
    jmp .cnvrt
.ten0:    inc si
    cmp cx, 1
    jb .return
    sub cx, 1
    inc byte [si]
    jmp .cnvrt
.return:
    ret

printrm:
    mov al, [si]
    test al, al
    jz .return
    call charrm
    inc si
    jmp printrm
.return:
    ret

charrm:             ;char must be in al
    mov bx, 7
    mov ah, 0xE
    int 10h
    ret

; .bestmode:    ;preference is width > height > color
    ; mov bx, [VBEModeInfo.xresolution]
    ; cmp bx, [.width]
    ; ja .switchmode
    ; jb .searchmodes
    ; mov bx, [VBEModeInfo.yresolution]
    ; cmp bx, [.height]
    ; ja .switchmode
    ; jb .searchmodes
    ; mov bl, [VBEModeInfo.bitsperpixel]
    ; cmp bl, [.color]
    ; jb .searchmodes
; .switchmode:
    ; mov cx, [.currentmode]
    ; mov [.mode], cx
    ; mov bx, [VBEModeInfo.xresolution]
    ; mov [.width], bx
    ; mov bx, [VBEModeInfo.yresolution]
    ; mov [.height], bx
    ; mov bl, [VBEModeInfo.bitsperpixel]
    ; mov [.color], bl
    ; jmp .searchmodes

; .mode dw 0
; .color db 0
; .height dw 0
; .width dw 0
