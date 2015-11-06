/* note these headers are all provided by newlib - you don't need to provide them */
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/fcntl.h>
#include <sys/times.h>
#include <sys/errno.h>
#include <sys/time.h>
#include <stdio.h>

#include <errno.h>
#undef errno
extern int errno;

#define SYS_BRK 45
#define SYS_CHDIR 12
#define SYS_CLONE 120
    #define CLONE_VM 0x100
    #define CLONE_FS 0x200
    #define CLONE_FILES 0x400
#define SYS_CLOSE 6
#define SYS_CLOCK_GETTIME 265
#define SYS_DUP 41
#define SYS_EXECVE 11
#define SYS_EXIT 1
#define SYS_FPATH 3001
#define SYS_FSTAT 28
#define SYS_FSYNC 118
#define SYS_FTRUNCATE 93
#define SYS_LINK 9
#define SYS_LSEEK 19
#define SYS_NANOSLEEP 162
#define SYS_OPEN 5
#define SYS_READ 3
#define SYS_UNLINK 10
#define SYS_WRITE 4
#define SYS_YIELD 158

uint syscall(uint a, uint b, uint c, uint d);
