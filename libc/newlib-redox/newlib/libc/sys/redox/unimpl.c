#include "common.h"

#include <stdio.h>

#define UNIMPL(error) { \
    errno = error; \
    fprintf(stderr, "unimplemented: %s: %s\n", __func__, strerror(error)); \
    return -1; \
}

int access(const char * path, int amode){
    UNIMPL(EACCES);
}

int chmod(const char * path, mode_t mode) {
    UNIMPL(EACCES);
}

int dup2(int oldfd, int newfd) {
    UNIMPL(EBADF);
}

int fcntl(int file, int cmd, ...){
    UNIMPL(EACCES);
}

int fstat(int file, struct stat *st) {
    st->st_mode = S_IFREG;
    return 0;
}

struct hostent * gethostbyname(const char * name) {
    return (struct hostent *) NULL;
}

int getdtablesize() {
    return 65536;
}

struct group * getgrnam(const char * name){
    return (struct group *) NULL;
}

struct group * getgrgid(gid_t gid){
    return (struct group *) NULL;
}

struct passwd * getpwnam(const char * name){
    return (struct passwd *) NULL;
}

struct passwd * getpwuid(uid_t uid){
    return (struct passwd *) NULL;
}

gid_t getegid() {
    return 0;
}

uid_t geteuid() {
    return 0;
}

gid_t getgid() {
    return 0;
}

uid_t getuid() {
    return 0;
}

int ioctl(int file, int request, ...) {
    UNIMPL(EINVAL);
}

int kill(int pid, int sig) {
    UNIMPL(EINVAL);
}

int mkdir(const char * path, mode_t mode) {
    UNIMPL(EACCES);
}

int pipe(int pipefd[2]) {
    UNIMPL(EINVAL);
}

int rmdir(const char * path){
    UNIMPL(EACCES);
}

int setgid(gid_t gid) {
    UNIMPL(EINVAL);
}

int setuid(uid_t uid) {
    UNIMPL(EINVAL);
}

int stat(const char *__restrict path, struct stat *__restrict sbuf) {
    sbuf->st_mode = S_IFREG;
    return 0;
}

long sysconf(int name) {
    UNIMPL(EINVAL);
}

clock_t times(struct tms * buf) {
    UNIMPL(EINVAL);
}

mode_t umask(mode_t mask) {
    return 0777;
}

int utime(const char * filename, const struct utimbuf * times) {
    UNIMPL(EACCES);
}

pid_t wait(int * status) {
    UNIMPL(ECHILD);
}

pid_t waitpid(pid_t pid, int * status, int options) {
    UNIMPL(ECHILD);
}
