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

#define SYS_DEBUG 0
#define SYS_EXIT 1
#define SYS_FORK 2
#define SYS_READ 3
#define SYS_WRITE 4
#define SYS_OPEN 5
#define SYS_CLOSE 6
#define SYS_TIME 13
#define SYS_LSEEK 19
#define SYS_FSTAT 28
#define SYS_YIELD 158

#define SYS_TRIGGER 1000
#define SYS_WINDOW_CREATE 1001
#define SYS_WINDOW_DESTROY 1002
#define SYS_READ_TO_END 1003
#define SYS_ALLOC 1004
#define SYS_REALLOC 1005
#define SYS_REALLOC_INPLACE 1006
#define SYS_UNALLOC 1007

uint syscall(uint eax, uint ebx, uint ecx, uint edx) {
    asm volatile("int $0x80"
        : "=a"(eax)
        : "a"(eax), "b"(ebx), "c"(ecx), "d"(edx)
        : "memory");

    return eax;
}

void _exit(int code){
    syscall(SYS_EXIT, (uint)code, 0, 0);
}

int close(int file){
    return (int)syscall(SYS_CLOSE, (uint)file, 0, 0);
}

char *__env[1] = { 0 };
char **environ = __env;

int execve(char *name, char **argv, char **env) {
  errno = ENOMEM;
  return -1;
}

int fork(void) {
  errno = EAGAIN;
  return -1;
}

int fstat(int file, struct stat *st) {
  st->st_mode = S_IFCHR;
  return 0;
}

int getpid() {
  return 1;
}

int isatty(int file) {
  return 1;
}

int kill(int pid, int sig) {
  errno = EINVAL;
  return -1;
}

int link(char *old, char *new) {
  errno = EMLINK;
  return -1;
}

int lseek(int file, int ptr, int dir) {
  return (int)syscall(SYS_LSEEK, (uint)file, (uint)ptr, (uint)dir);
}

int open(const char * file, int flags, ...) {
  return (int)syscall(SYS_OPEN, (uint)file, (uint)flags, 0);
}

int read(int file, char *ptr, int len) {
  return (int)syscall(SYS_READ, (uint)file, (uint)ptr, (uint)len);
}

caddr_t sbrk(int incr) {
  extern char _end;		/* Defined by the linker */
  static char *heap_end;
  char *prev_heap_end;
  if (heap_end == 0) {
    heap_end = &_end;
  }
  prev_heap_end = heap_end;
  heap_end += incr;
  return (caddr_t) prev_heap_end;
}

int stat(const char *__restrict path, struct stat *__restrict sbuf) {
  sbuf->st_mode = S_IFCHR;
  return 0;
}

clock_t times(struct tms *buf) {
  return -1;
}

int unlink(char *name) {
  errno = ENOENT;
  return -1;
}

int wait(int *status) {
  errno = ECHILD;
  return -1;
}

int write(int file, char *ptr, int len) {
    return (int)syscall(SYS_WRITE, (uint)file, (uint)ptr, (uint)len);
}
