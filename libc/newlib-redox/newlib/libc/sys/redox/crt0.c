#include <fcntl.h>

extern void exit(int code);
extern int main(int argc, char ** argv);

void _start() {
    exit(main(0, 0));
}
