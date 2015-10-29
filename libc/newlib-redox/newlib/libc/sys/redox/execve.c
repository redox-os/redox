#ifndef _NO_EXECVE

/* execve.c */

/* This and the other exec*.c files in this directory require 
   the target to provide the _execve syscall.  */


#include <unistd.h>


int
_DEFUN(execve, (path, argv, envp),
      const char *path _AND
      char * const argv[] _AND
      char * const envp[])
{
  return _execve (path, argv, envp);
}

#endif /* !_NO_EXECVE  */
