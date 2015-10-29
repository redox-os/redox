#ifndef _NO_EXECVE

/* execvp.c */

/* This and the other exec*.c files in this directory require 
   the target to provide the _execve syscall.  */

#include <_ansi.h>
#include <stdlib.h>
#include <unistd.h>
#include <dirent.h>
#include <string.h>
#include <errno.h>
#include <ctype.h>

#define PATH_DELIM ':'

/*
 * Copy string, until c or <nul> is encountered.
 * NUL-terminate the destination string (s1).
 */

static char *
_DEFUN (strccpy, (s1, s2, c),
	char *s1 _AND
	char *s2 _AND
	char c)
{
  char *dest = s1;

  while (*s2 && *s2 != c)
    *s1++ = *s2++;
  *s1 = 0;

  return dest;
}

int
_DEFUN (execvp, (file, argv),
	_CONST char *file _AND
	char * _CONST argv[])
{
  char *path = getenv ("PATH");
  char buf[MAXNAMLEN];

  /* If $PATH doesn't exist, just pass FILE on unchanged.  */
  if (!path)
    return execv (file, argv);

  /* If FILE contains a directory, don't search $PATH.  */
  if (strchr (file, '/')
      )
    return execv (file, argv);

  while (*path)
    {
      strccpy (buf, path, PATH_DELIM);
      /* An empty entry means the current directory.  */
      if (*buf != 0 && buf[strlen(buf) - 1] != '/')
	strcat (buf, "/");
      strcat (buf, file);
      if (execv (buf, argv) == -1 && errno != ENOENT)
	return -1;
      while (*path && *path != PATH_DELIM)
	path++;
      if (*path == PATH_DELIM)
	path++;			/* skip over delim */
    }

  return -1;
}

#endif /* !_NO_EXECVE  */
