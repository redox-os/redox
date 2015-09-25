#include <stdlib.h>
#include <stdio.h>
#include <sys/time.h>

int main(int argc, char ** argv){
    printf("Test\n");
    struct timeval tv;
    if(gettimeofday(&tv, NULL) == 0){
        printf("Gettimeofday %d %d\n", tv.tv_sec, tv.tv_usec);
        void* test = malloc(1024*1024);
        if(test > 0){
            printf("Malloc %x\n", test);
            free(test);
            printf("Free\n");
        }else{
            printf("Malloc Failed\n");
        }
    }else{
        printf("Gettimeofday Failed\n");
    }
    return 0;
}
