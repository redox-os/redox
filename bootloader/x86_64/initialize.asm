SECTION .text
USE16

initialize:
.fpu: ;enable fpu
    mov eax, cr0
    and al, 11110011b
    or al, 00100010b
    mov cr0, eax
    mov eax, cr4
    or eax, 0x200
    mov cr4, eax
    fninit
    ret

.sse: ;enable sse
    mov eax, cr4
    or ax, 0000011000000000b
    mov cr4, eax
    ret

;PIT Frequency
;If using nanoseconds, to minimize drift, one should find a frequency as close to an integer nanosecond value in wavelength
;Divider    Hz                                Nanoseconds                            Properties
;2685        444.38795779019242706393        2250286.00003631746492922946        Best For Context Switching
;5370        222.19397889509621353196        4500572.00007263492985856020
;21029       56.73981961418358774390         17624306.99991199998882825455
;23714       50.31549576902532962244         19874592.99994831745375667118
;26399       45.19798729749864262535         22124878.99998463491868476373
;29084       41.02536331545408701233         24375165.00002095238361424615
;31769       37.55804925136663623868         26625451.00005726984854313455
;34454       34.63115071302799868423         28875737.00009358731347639618
;50113       23.80982313305263437963         41999471.99993295237244784676
;52798       22.59899364874932131267         44249757.99996926983737931766
;55483       21.50535599492937776736         46500044.00000558730230583335        Lowest Drift
;58168       20.51268165772704350616         48750330.00004190476724037528
;60853       19.60760630809765610021         51000616.00007822223218031738

.pit:
    ;initialize the PIT
    mov ax, 2685 ;this is the divider for the PIT
    out 0x40, al
    rol ax, 8
    out 0x40, al
    ;DISABLED ;enable rtc interrupt
    ;mov al, 0xB
    ;out 0x70, al
    ;rol ax, 8
    ;in al, 0x71
    ;rol ax, 8
    ;out 0x70, al
    ;rol ax, 8
    ;or al, 0x40
    ;out 0x71, al
    ret

.pic:    ;sets up IRQs at int 20-2F
    mov al, 0x11
    out 0x20, al
    out 0xA0, al
    mov al, 0x20    ;IRQ0 vector
    out 0x21, al
    mov al, 0x28    ;IRQ8 vector
    out 0xA1, al
    mov al, 4
    out 0x21, al
    mov al, 2
    out 0xA1, al
    mov al, 1
    out 0x21, al
    out 0xA1, al
    xor al, al        ;no IRQ masks
    out 0x21, al
    out 0xA1, al
    mov al, 0x20    ;reset PIC's
    out 0xA0, al
    out 0x20, al
    ret
