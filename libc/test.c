#include <stdlib.h>
#include <stdio.h>

int main(int argc, char ** argv){
    printf("Test\n");
    void* test = malloc(1024*1024);
    if(test > 0){
        printf("Malloc\n");
        free(test);
        printf("Free\n");
    }else{
        printf("Malloc Failed\n");
    }
    return 0;
}
