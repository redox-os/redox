#include "common.h"

uint syscall(uint a, uint b, uint c, uint d){
    asm volatile("int $0x80"
        : "=a"(a)
        : "a"(a), "b"(b), "c"(c), "d"(d)
        : "memory");

    return a;
}
