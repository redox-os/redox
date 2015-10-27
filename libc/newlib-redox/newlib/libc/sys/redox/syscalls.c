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
#define SYS_CLOSE 6
#define SYS_DUP 41
#define SYS_EXECVE 11
#define SYS_EXIT 1
#define SYS_FORK 2
#define SYS_FPATH 3001
#define SYS_FSTAT 28
#define SYS_FSYNC 118
#define SYS_GETTIMEOFDAY 78
#define SYS_LINK 9
#define SYS_LSEEK 19
#define SYS_OPEN 5
#define SYS_READ 3
#define SYS_UNLINK 10
#define SYS_WRITE 4
#define SYS_YIELD 158

uint syscall(uint a, uint b, uint c, uint d) {
    asm volatile("int $0x80"
        : "=a"(a)
        : "a"(a), "b"(b), "c"(c), "d"(d)
        : "memory");

    return a;
}

void _exit(int code){
    syscall(SYS_EXIT, (uint)code, 0, 0);
}

int chdir(const char *path){
    return (int)syscall(SYS_CHDIR, (uint)path, 0, 0);
}

int close(int file){
    return (int)syscall(SYS_CLOSE, (uint)file, 0, 0);
}

int dup(int file){
    return (int)syscall(SYS_DUP, (uint)file, 0, 0);
}

int execve(const char *name, const char **argv, const char **env) {
    return (int)syscall(SYS_EXECVE, (uint)name, (uint)argv, (uint)env);
}

int fork(void) {
    return (int)syscall(SYS_FORK, 0, 0, 0);
}

int fpath(int file, char *ptr, int len) {
    return (int)syscall(SYS_FPATH, (uint)ptr, (uint)len, 0);
}

int fstat(int file, struct stat *st) {
    st->st_mode = S_IFCHR;
    return 0;
}

int fsync(int file) {
    return (int)syscall(SYS_FSYNC, (uint)file, 0, 0);
}

int getpid() {
    return 1;
}

int gettimeofday(struct timeval *__restrict tv, void *__restrict tz){
    return (int)syscall(SYS_GETTIMEOFDAY, (uint)tv, (uint)tz, 0);
}

int isatty(int file) {
    return 1;
}

int kill(int pid, int sig) {
    errno = EINVAL;
    return -1;
}

int link(const char *old, const char *new) {
    return (int)syscall(SYS_LINK, (uint)old, (uint)new, 0);
}

int lseek(int file, int ptr, int dir) {
    return (int)syscall(SYS_LSEEK, (uint)file, (uint)ptr, (uint)dir);
}

int open(const char *file, int flags, ...) {
    return (int)syscall(SYS_OPEN, (uint)file, (uint)flags, 0);
}

int read(int file, char *ptr, int len) {
    return (int)syscall(SYS_READ, (uint)file, (uint)ptr, (uint)len);
}

void *sbrk(ptrdiff_t increment){
    char * curr_brk = (char *)syscall(SYS_BRK, 0, 0, 0);
    char * new_brk = (char *)syscall(SYS_BRK, (uint)(curr_brk + increment), 0, 0);
    if (new_brk != curr_brk + increment){
        return (void *) -1;
    }
    return curr_brk;
}

int stat(const char *__restrict path, struct stat *__restrict sbuf) {
    sbuf->st_mode = S_IFCHR;
    return 0;
}

clock_t times(struct tms *buf) {
    return -1;
}

int unlink(const char *name) {
    return (int)syscall(SYS_UNLINK, (uint)name, 0, 0);
}

int wait(int *status) {
    errno = ECHILD;
    return -1;
}

int write(int file, const char *ptr, int len) {
    return (int)syscall(SYS_WRITE, (uint)file, (uint)ptr, (uint)len);
}
