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

#include <sys/dirent.h>

DIR *opendir(const char * path) {
    int fd = open(path, O_RDONLY);

    if(fd >= 0){
        DIR * dir = (DIR *)calloc(sizeof(DIR), 1);
        dir->dd_fd = fd;
        return dir;
    }

    return NULL;
}

struct dirent *readdir(DIR * dir){
    if(dir){
        //TODO: Speed improvements
        int i;
        for(i = 0; i < 4096; ++i){
            if(read(dir->dd_fd, &(dir->dd_ent.d_name[i]), 1) > 0){
                if(dir->dd_ent.d_name[i] == '\n'){
                    break;
                }
            }else{
                break;
            }
        }
        dir->dd_ent.d_name[i] = '\0';

        if(i > 0){
            return &(dir->dd_ent);
        }
    }

    return NULL;
}

void rewinddir(DIR * dir){
    if(dir){
        lseek(dir->dd_fd, 0, 0);
    }
}

int closedir(DIR * dir){
    if(dir){
        close(dir->dd_fd);
        free(dir);
        return 0;
    }

    return -1;
}
