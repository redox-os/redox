use boxed::Box;
use core::mem;
use fmt;
use io::{Result, Read, Write};
use os::unix::io::{AsRawFd, FromRawFd, RawFd};
use ops::DerefMut;
use string::{String, ToString};
use core_collections::borrow::ToOwned;
use vec::Vec;

use io::Error;
use system::syscall::{sys_clone, sys_close, sys_dup, sys_execve, sys_exit, sys_pipe2, sys_read, sys_write, sys_waitpid, CLONE_VM, CLONE_VFORK, CLONE_SUPERVISE};
use system::error::Error as SysError;

pub struct ExitStatus {
    status: usize,
}

impl ExitStatus {
    pub fn success(&self) -> bool {
        self.status == 0
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.status as i32)
    }
}

pub struct ChildStdin {
    fd: usize,
}

impl Write for ChildStdin {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        sys_write(self.fd, buf).map_err(|x| Error::from_sys(x))
    }
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

impl Drop for ChildStdin {
    fn drop(&mut self) {
        let _ = sys_close(self.fd);
    }
}

pub struct ChildStdout {
    fd: usize,
}

impl AsRawFd for ChildStdout {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl Read for ChildStdout {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys_read(self.fd, buf).map_err(|x| Error::from_sys(x))
    }
}

impl Drop for ChildStdout {
    fn drop(&mut self) {
        let _ = sys_close(self.fd);
    }
}

pub struct ChildStderr {
    fd: usize,
}

impl Read for ChildStderr {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        sys_read(self.fd, buf).map_err(|x| Error::from_sys(x))
    }
}

impl Drop for ChildStderr {
    fn drop(&mut self) {
        let _ = sys_close(self.fd);
    }
}

pub struct Child {
    pid: usize,
    pub stdin: Option<ChildStdin>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
}

impl Child {
    pub fn id(&self) -> u32 {
        self.pid as u32
    }

    pub fn wait(&mut self) -> Result<ExitStatus> {
        let mut status: usize = 0;
        sys_waitpid(self.pid, &mut status, 0).map(|_| ExitStatus { status: status }).map_err(|x| Error::from_sys(x))
    }
}

pub struct Command {
    pub path: String,
    pub args: Vec<String>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{:?}", self.path));
        for arg in &self.args {
            try!(write!(f, " {:?}", arg));
        }
        Ok(())
    }
}

impl Command {
    pub fn new(path: &str) -> Command {
        Command {
            path: path.to_owned(),
            args: Vec::new(),
            stdin: Stdio::inherit(),
            stdout: Stdio::inherit(),
            stderr: Stdio::inherit(),
        }
    }

    pub fn arg(&mut self, arg: &str) -> &mut Command {
        self.args.push(arg.to_owned());
        self
    }

    pub fn stdin(&mut self, cfg: Stdio) -> &mut Command {
        self.stdin = cfg;
        self
    }

    pub fn stdout(&mut self, cfg: Stdio) -> &mut Command {
        self.stdout = cfg;
        self
    }

    pub fn stderr(&mut self, cfg: Stdio) -> &mut Command {
        self.stderr = cfg;
        self
    }

    pub fn spawn(&mut self) -> Result<Child> {
        self.exec(CLONE_VM | CLONE_VFORK)
    }

    /// Spawn this command as a supervised process.
    ///
    /// This means that the system calls will block the process, until being handled by the
    /// parrent. Handling can be done by calling `.id()`, and then using `sys_supervise` to start
    /// supervising this process. Refer to the respective documentation for more information.
    pub fn spawn_supervise(&mut self) -> Result<Child> {
        self.exec(CLONE_VM | CLONE_SUPERVISE)
    }

    fn exec(&mut self, flags: usize) -> Result<Child> {
        let mut res = Box::new(0);

        let path_c = if self.path.contains('/') {
            self.path.to_owned() + "\0"
        } else {
            let mut path_env = super::env::var("PATH").unwrap_or(".".to_string());

            if ! path_env.ends_with('/') {
                path_env.push('/');
            }

            path_env.push_str(&self.path);

            path_env + "\0"
        };

        let mut args_vec: Vec<String> = Vec::new();
        for arg in self.args.iter() {
            args_vec.push(arg.to_owned() + "\0");
        }

        let mut args_c: Vec<*const u8> = Vec::new();
        for arg_vec in args_vec.iter() {
            args_c.push(arg_vec.as_ptr());
        }
        args_c.push(0 as *const u8);

        let child_res = res.deref_mut() as *mut usize;
        let child_stderr = self.stderr.inner;
        let child_stdout = self.stdout.inner;
        let child_stdin = self.stdin.inner;
        let child_code = Box::new(move || -> Result<usize> {
            match child_stderr {
                StdioType::Piped(read, write) => {
                    try!(sys_close(read).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(2).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(write).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(write).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Raw(fd) => {
                    try!(sys_close(2).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(fd).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Null => {
                    try!(sys_close(2).map_err(|x| Error::from_sys(x)));
                },
                _ => ()
            }

            match child_stdout {
                StdioType::Piped(read, write) => {
                    try!(sys_close(read).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(1).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(write).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(write).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Raw(fd) => {
                    try!(sys_close(1).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(fd).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Null => {
                    try!(sys_close(1).map_err(|x| Error::from_sys(x)));
                },
                _ => ()
            }

            match child_stdin {
                StdioType::Piped(read, write) => {
                    try!(sys_close(write).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(0).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(read).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(read).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Raw(fd) => {
                    try!(sys_close(0).map_err(|x| Error::from_sys(x)));
                    try!(sys_dup(fd).map_err(|x| Error::from_sys(x)));
                    try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                },
                StdioType::Null => {
                    try!(sys_close(0).map_err(|x| Error::from_sys(x)));
                },
                _ => ()
            }

            unsafe { sys_execve(path_c.as_ptr(), args_c.as_ptr()) }.map_err(|x| Error::from_sys(x))
        });

        match unsafe { sys_clone(flags) } {
            Ok(0) => {
                let error = child_code();

                unsafe { *child_res = SysError::mux(error.map_err(|x| x.into_sys())); }

                loop {
                    let _ = sys_exit(127);
                }
            },
            Ok(pid) => {
                // Must forget child_code to prevent double free
                mem::forget(child_code);
                if let Err(err) = SysError::demux(*res) {
                    Err(Error::from_sys(err))
                } else {
                    Ok(Child {
                        pid: pid,
                        stdin: match self.stdin.inner {
                            StdioType::Piped(read, write) => {
                                try!(sys_close(read).map_err(|x| Error::from_sys(x)));
                                Some(ChildStdin {
                                    fd: write
                                })
                            },
                            StdioType::Raw(fd) => {
                                try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                                None
                            },
                            _ => None
                        },
                        stdout: match self.stdout.inner {
                            StdioType::Piped(read, write) => {
                                try!(sys_close(write).map_err(|x| Error::from_sys(x)));
                                Some(ChildStdout {
                                    fd: read
                                })
                            },
                            StdioType::Raw(fd) => {
                                try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                                None
                            },
                            _ => None
                        },
                        stderr: match self.stderr.inner {
                            StdioType::Piped(read, write) => {
                                try!(sys_close(write).map_err(|x| Error::from_sys(x)));
                                Some(ChildStderr {
                                    fd: read
                                })
                            },
                            StdioType::Raw(fd) => {
                                try!(sys_close(fd).map_err(|x| Error::from_sys(x)));
                                None
                            },
                            _ => None
                        }
                    })
                }
            }
            Err(err) => Err(Error::from_sys(err))
        }
    }
}

#[derive(Copy, Clone)]
enum StdioType {
    Piped(usize, usize),
    Raw(usize),
    Inherit,
    Null,
}

pub struct Stdio {
    inner: StdioType,
}

impl Stdio {
    pub fn piped() -> Stdio {
        let mut fds = [0; 2];
        if unsafe { sys_pipe2(fds.as_mut_ptr(), 0).is_ok() } {
            Stdio {
                inner: StdioType::Piped(fds[0], fds[1])
            }
        } else {
            Stdio::null()
        }
    }

    pub fn inherit() -> Stdio {
        Stdio {
            inner: StdioType::Inherit
        }
    }

    pub fn null() -> Stdio {
        Stdio {
            inner: StdioType::Null
        }
    }
}

impl FromRawFd for Stdio {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Stdio {
            inner: StdioType::Raw(fd)
        }
    }
}

pub fn exit(code: i32) -> ! {
    loop {
        let _ = sys_exit(code as usize);
    }
}
