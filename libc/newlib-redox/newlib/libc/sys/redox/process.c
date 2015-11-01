#include "common.h"

int chdir(const char *path){
    return (int)syscall(SYS_CHDIR, (uint)path, 0, 0);
}

int clone() {
    return (int)syscall(SYS_CLONE, 0, 0, 0);
}

void _exit(int code){
    syscall(SYS_EXIT, (uint)code, 0, 0);
}

int _execve(const char *name, const char **argv, const char **env) {
    return (int)syscall(SYS_EXECVE, (uint)name, (uint)argv, (uint)env);
}

char * getcwd(char * buf, size_t size) {
    char * cwd = NULL;

    int file = open("", O_RDONLY);
    if(file >= 0){
        if(!buf){
            if(size == 0){
                size = 4096;
            }
            buf = (char *)calloc(size, 1);

            if(fpath(file, buf, size) >= 0){
                cwd = buf;
            }else{
                free(buf);
            }
        }else{
            memset(buf, 0, size);
            if(fpath(file, buf, size) >= 0){
                cwd = buf;
            }
        }
        close(file);
    }

    return cwd;
}

int fork() {
    return (int)syscall(SYS_FORK, 0, 0, 0);
}

void * sbrk(ptrdiff_t increment){
    char * curr_brk = (char *)syscall(SYS_BRK, 0, 0, 0);
    char * new_brk = (char *)syscall(SYS_BRK, (uint)(curr_brk + increment), 0, 0);
    if (new_brk != curr_brk + increment){
        return (void *) -1;
    }
    return curr_brk;
}
