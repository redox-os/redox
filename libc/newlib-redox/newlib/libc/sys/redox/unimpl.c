#include "common.h"

int access(const char * path, int amode){
    errno = EACCES;
    return -1;
}

int fcntl(int file, int cmd, ...){
    errno = EACCES;
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

int mkdir(const char * path, mode_t mode) {
    errno = EACCES;
    return -1;
}

int rmdir(const char * path){
    errno = EACCES;
    return -1;
}

int stat(const char *__restrict path, struct stat *__restrict sbuf) {
    sbuf->st_mode = S_IFCHR;
    return 0;
}

/*
clock_t times(struct tms *buf) {
    return -1;
}
*/

/*
int wait(int *status) {
    errno = ECHILD;
    return -1;
}
*/
