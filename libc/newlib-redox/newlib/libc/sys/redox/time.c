#include "common.h"

int clock_gettime(clockid_t clk_id, struct timespec * tp){
    return (int)syscall(SYS_CLOCK_GETTIME, (uint)clk_id, (uint)tp, 0);
}

int gettimeofday(struct timeval *__restrict tv, void *__restrict tz){
    if(tv){
        struct timespec tp;
        if(clock_gettime(CLOCK_REALTIME, &tp) == 0){
            tv->tv_sec = tp.tv_sec;
            tv->tv_usec = tp.tv_nsec / 1000;
        }
    }

    errno = EINVAL;
    return -1;
}

int nanosleep(const struct timespec * req, struct timespec * rem){
    return (int)syscall(SYS_NANOSLEEP, (uint)req, (uint)rem, 0);
}
