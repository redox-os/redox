#include "common.h"

int gettimeofday(struct timeval *__restrict tv, void *__restrict tz){
    return (int)syscall(SYS_GETTIMEOFDAY, (uint)tv, (uint)tz, 0);
}


int nanosleep(const struct timespec * req, struct timespec * rem){
    return (int)syscall(SYS_NANOSLEEP, (uint)req, (uint)rem, 0);
}
