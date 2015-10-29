#ifndef _NO_EXECVE

/* execv.c */

/* This and the other exec*.c files in this directory require 
   the target to provide the _execve syscall.  */

#include <_ansi.h>
#include <unistd.h>

/* Only deal with a pointer to environ, to work around subtle bugs with shared
   libraries and/or small data systems where the user declares his own
   'environ'.  */
static char ***p_environ = &environ;

int
_DEFUN (execv, (path, argv), 
	const char *path _AND
	char * const argv[])
{
  return _execve (path, (char * _CONST *) argv, *p_environ);
}

#endif /* !_NO_EXECVE  */
