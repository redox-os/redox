#include <stdlib.h>
#include <stdio.h>
#include <dirent.h>
#include <sys/time.h>

int main(int argc, char ** argv){
    printf("Test %d\n", argc);
    int i;
    for(i = 0; i < argc; i++){
        printf("%d: %s\n", i, argv[i]);
    }
    struct timespec tp;
    if(clock_gettime(CLOCK_REALTIME, &tp) == 0){
        printf("clock_gettime %d %d\n", tp.tv_sec, tp.tv_nsec);
        void* test = malloc(1024*1024);
        if(test > 0){
            printf("Malloc %x\n", test);
            free(test);
            printf("Free\n");

            DIR * dir = opendir("file:///");
            if (dir != NULL) {
                struct dirent * ent;
                while ((ent = readdir(dir)) != NULL) {
                    printf("%s\n", ent->d_name);
                }
                closedir(dir);

        	    pid_t pid = fork();
                if(pid > 0){
                    printf("Fork Parent %d = %d\n", getpid(), pid);
                    int status = 0;
                    if(waitpid(pid, &status, 0) >= 0){
                        printf("waitpid status %d\n", status);
                    }else{
                        printf("waitpid failed\n");
                    }
                }else if(pid == 0){
                    printf("Fork Child %d = %d\n", getpid(), pid);
                    _exit(123);
                } else {
                    printf("Fork Failed %d = %d\n", getpid(), pid);
                }
            }else{
                printf("Opendir Failed\n");
            }
        } else {
            printf("Malloc Failed\n");
        }
    } else {
        printf("clock_gettime Failed\n");
    }
    return 0;
}
