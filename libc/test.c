#include <stdlib.h>
#include <stdio.h>

int main(int argc, char ** argv){
    puts("Test");
    struct timeval tv;
    if(gettimeofday(&tv, NULL) == 0){
        puts("Gettimeofday");
        void* test = malloc(1024*1024);
        if(test > 0){
            puts("Malloc");
            free(test);
            puts("Free");
        }else{
            puts("Malloc Failed");
        }
    }else{
        puts("Gettimeofday Failed");
    }
    return 0;
}
