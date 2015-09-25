extern char ** environ;
extern void exit(int code);
extern int main(int argc, char ** argv, char ** envp);

void _start(int args) {
    int * params = &args-1;
    int argc = *params;
    char ** argv = (char **)(params + 1);
    environ = argv+argc+1;
    
    exit(main(argc, argv, environ));
}
