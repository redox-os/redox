#include <stdlib.h>
#include <stdio.h>
#include <sys/time.h>

int main(int argc, char ** argv){
    printf("Test %d\n", argc);
    int i;
    for(i = 0; i < argc; i++){
        printf("%d: %s\n", i, argv[i]);
    }
    struct timeval tv;
    if(gettimeofday(&tv, NULL) == 0){
        printf("Gettimeofday %d %d\n", tv.tv_sec, tv.tv_usec);
        void* test = malloc(1024*1024);
        if(test > 0){
            printf("Malloc %x\n", test);
            free(test);
            printf("Free\n");

	    pid_t pid = fork();
            if(pid == 0){
                printf("Fork Parent\n");
            }else if(pid > 0){
                printf("Fork Child %d\n", pid);
            } else {
                printf("Fork Failed\n");
            }
        } else {
            printf("Malloc Failed\n");
        }
    } else {
        printf("Gettimeofday Failed\n");
    }
    return 0;
}
