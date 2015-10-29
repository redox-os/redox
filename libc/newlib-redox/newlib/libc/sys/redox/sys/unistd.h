#ifndef _UNISTD_H
#define _UNISTD_H

//#include <features.h>

#ifdef __cplusplus
extern "C" {
#endif

#include <_ansi.h>
#include <sys/types.h>
#include <sys/_types.h>
#define __need_size_t
#define __need_ptrdiff_t
#include <stddef.h>

extern char **environ;

void	_EXFUN(_exit, (int __status ) _ATTRIBUTE ((__noreturn__)));

int	_EXFUN(access,(const char *__path, int __amode ));
unsigned  _EXFUN(alarm, (unsigned __secs ));
int     _EXFUN(chdir, (const char *__path ));
int     _EXFUN(chmod, (const char *__path, mode_t __mode ));
int     _EXFUN(chown, (const char *__path, uid_t __owner, gid_t __group ));
int     _EXFUN(chroot, (const char *__path ));
int     _EXFUN(close, (int __fildes ));
char    _EXFUN(*ctermid, (char *__s ));
char    _EXFUN(*cuserid, (char *__s ));
int     _EXFUN(dup, (int __fildes ));
int     _EXFUN(dup2, (int __fildes, int __fildes2 ));
int     _EXFUN(execl, (const char *__path, const char *, ... ));
int     _EXFUN(execle, (const char *__path, const char *, ... ));
int     _EXFUN(execlp, (const char *__file, const char *, ... ));
int     _EXFUN(execv, (const char *__path, char * const __argv[] ));
int     _EXFUN(execve, (const char *__path, char * const __argv[], char * const __envp[] ));
int     _EXFUN(execvp, (const char *__file, char * const __argv[] ));
int     _EXFUN(fchdir, (int __fildes));
int     _EXFUN(fchmod, (int __fildes, mode_t __mode ));
int     _EXFUN(fchown, (int __fildes, uid_t __owner, gid_t __group ));
pid_t   _EXFUN(fork, (void ));
long    _EXFUN(fpathconf, (int __fd, int __name ));
int     _EXFUN(fsync, (int __fd));
int     _EXFUN(ftruncate, (int __fd, off_t __length));
char    _EXFUN(*getcwd, (char *__buf, size_t __size ));
int	_EXFUN(getdomainname ,(char *__name, size_t __len));
gid_t   _EXFUN(getegid, (void ));
uid_t   _EXFUN(geteuid, (void ));
gid_t   _EXFUN(getgid, (void ));
int     _EXFUN(getgroups, (int __gidsetsize, gid_t __grouplist[] ));
int 	_EXFUN(__gethostname, (char *__name, size_t __len));
char    _EXFUN(*getlogin, (void ));
#if defined(_POSIX_THREAD_SAFE_FUNCTIONS)
int _EXFUN(getlogin_r, (char *name, size_t namesize) );
#endif
char 	_EXFUN(*getpass, (__const char *__prompt));
int  _EXFUN(getpagesize, (void));
pid_t   _EXFUN(getpgid, (pid_t));
pid_t   _EXFUN(getpgrp, (void ));
pid_t   _EXFUN(getpid, (void ));
pid_t   _EXFUN(getppid, (void ));
uid_t   _EXFUN(getuid, (void ));
char *	_EXFUN(getusershell, (void));
char    _EXFUN(*getwd, (char *__buf ));
int     _EXFUN(isatty, (int __fildes ));
int     _EXFUN(lchown, (const char *__path, uid_t __owner, gid_t __group ));
int     _EXFUN(link, (const char *__path1, const char *__path2 ));
int	_EXFUN(nice, (int __nice_value ));
off_t   _EXFUN(lseek, (int __fildes, off_t __offset, int __whence ));
long    _EXFUN(pathconf, (const char *__path, int __name ));
int     _EXFUN(pause, (void ));
int     _EXFUN(pipe, (int __fildes[2] ));
ssize_t _EXFUN(pread, (int __fd, void *__buf, size_t __nbytes, off_t __offset));
ssize_t _EXFUN(pwrite, (int __fd, const void *__buf, size_t __nbytes, off_t __offset));
_READ_WRITE_RETURN_TYPE _EXFUN(read, (int __fd, void *__buf, size_t __nbyte ));
int     _EXFUN(readlink, (const char *path, char *buf, size_t bufsiz));
int     _EXFUN(rmdir, (const char *__path ));
void *  _EXFUN(sbrk,  (ptrdiff_t __incr));
int     _EXFUN(setegid, (gid_t __gid ));
int     _EXFUN(seteuid, (uid_t __uid ));
int     _EXFUN(setgid, (gid_t __gid ));
int     _EXFUN(setpgid, (pid_t __pid, pid_t __pgid ));
int     _EXFUN(setpgrp, (void ));
pid_t   _EXFUN(setsid, (void ));
int     _EXFUN(setuid, (uid_t __uid ));
unsigned _EXFUN(sleep, (unsigned int __seconds ));
void    _EXFUN(swab, (const void *, void *, ssize_t));
int     _EXFUN(symlink, (const char *oldpath, const char *newpath));
long    _EXFUN(sysconf, (int __name ));
pid_t   _EXFUN(tcgetpgrp, (int __fildes ));
int     _EXFUN(tcsetpgrp, (int __fildes, pid_t __pgrp_id ));
int     _EXFUN(truncate, (const char *, off_t __length));
char *  _EXFUN(ttyname, (int __fildes ));
int     _EXFUN(ttyname_r, (int __fildes, char *__buf, size_t __len));
int     _EXFUN(unlink, (const char *__path ));
int     _EXFUN(usleep, (unsigned int __useconds));
int     _EXFUN(vhangup, (void ));
_READ_WRITE_RETURN_TYPE _EXFUN(write, (int __fd, const void *__buf, size_t __nbyte ));

extern char *optarg;			/* getopt(3) external variables */
extern int optind, opterr, optopt;
int	 getopt(int, char * const [], const char *);
extern int optreset;			/* getopt(3) external variable */

#ifndef        _POSIX_SOURCE
pid_t   _EXFUN(vfork, (void ));

extern char *suboptarg;			/* getsubopt(3) external variable */
int	 getsubopt(char **, char * const *, char **);
#endif /* _POSIX_SOURCE */

/* Provide prototypes for most of the _<systemcall> names that are
   provided in newlib for some compilers.  */
int     _EXFUN(_close, (int __fildes ));
pid_t   _EXFUN(_fork, (void ));
pid_t   _EXFUN(_getpid, (void ));
int     _EXFUN(_link, (const char *__path1, const char *__path2 ));
off_t   _EXFUN(_lseek, (int __fildes, off_t __offset, int __whence ));
_READ_WRITE_RETURN_TYPE _EXFUN(_read, (int __fd, void *__buf, size_t __nbyte ));
void *  _EXFUN(_sbrk,  (size_t __incr));
int     _EXFUN(_unlink, (const char *__path ));
_READ_WRITE_RETURN_TYPE _EXFUN(_write, (int __fd, const void *__buf, size_t __nbyte ));
int     _EXFUN(_execve, (const char *__path, char * const __argv[], char * const __envp[] ));

#define	F_OK	0
#define	R_OK	4
#define	W_OK	2
#define	X_OK	1

# define	SEEK_SET	0
# define	SEEK_CUR	1
# define	SEEK_END	2

#include <sys/features.h>

#define STDIN_FILENO    0       /* standard input file descriptor */
#define STDOUT_FILENO   1       /* standard output file descriptor */
#define STDERR_FILENO   2       /* standard error file descriptor */

//#include <bits/environments.h>
//#include <bits/confname.h>

# define        MAXPATHLEN      1024

#ifdef __cplusplus
}
#endif
#endif /* _SYS_UNISTD_H */
