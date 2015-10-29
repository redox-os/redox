#include "common.h"

int close(int file){
    return (int)syscall(SYS_CLOSE, (uint)file, 0, 0);
}

int dup(int file){
    return (int)syscall(SYS_DUP, (uint)file, 0, 0);
}

int fpath(int file, char * buf, int len) {
    return (int)syscall(SYS_FPATH, (uint)buf, (uint)len, 0);
}

int fsync(int file) {
    return (int)syscall(SYS_FSYNC, (uint)file, 0, 0);
}

int ftruncate(int file, off_t len){
    return (int)syscall(SYS_FTRUNCATE, (uint)file, (uint)len, 0);
}

int lseek(int file, int ptr, int dir) {
    return (int)syscall(SYS_LSEEK, (uint)file, (uint)ptr, (uint)dir);
}

int link(const char *old, const char *new) {
    return (int)syscall(SYS_LINK, (uint)old, (uint)new, 0);
}

int open(const char *file, int flags, ...) {
    return (int)syscall(SYS_OPEN, (uint)file, (uint)flags, 0);
}

int read(int file, char *ptr, int len) {
    return (int)syscall(SYS_READ, (uint)file, (uint)ptr, (uint)len);
}

int unlink(const char *name) {
    return (int)syscall(SYS_UNLINK, (uint)name, 0, 0);
}

int write(int file, const char *ptr, int len) {
    return (int)syscall(SYS_WRITE, (uint)file, (uint)ptr, (uint)len);
}
