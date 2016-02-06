startup:
    xchg bx, bx
    in al, 0x92
    or al, 2
    out 0x92, al

    call memory_map

    call vesa

    call initialize.fpu
    call initialize.sse
    call initialize.pit
    call initialize.pic
