use libc::{self, winsize};
use std::fs::File;
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Child;
use std::time::Duration;
use std::{io, mem, ptr};
use std::{
    io::{PipeReader, PipeWriter},
    process::Command,
};

pub use std::os::unix::io::RawFd;

use crate::{Error, Result, wrap_io_err};

macro_rules! log_to_pty {
    ($logger:expr, $($arg:tt)+) => {
        if $logger.is_some() {
            use std::io::Write;
            let mut logfd = $logger.as_ref().unwrap().1.try_clone().unwrap();
            let _ = logfd.write(format!($($arg)+).as_bytes());
            let _ = logfd.write(&[b'\n']);
        } else {
            eprintln!($($arg)+);
        }
    };
}

pub(crate) use log_to_pty;

pub type PtyOut<'a> = Option<(&'a mut UnixSlavePty, &'a mut PipeWriter)>;

pub fn setup_pty() -> (
    Box<dyn Read + Send>,
    PipeReader,
    (UnixSlavePty, std::io::PipeWriter),
) {
    let pty_system = UnixPtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24, // Standard terminal size
            cols: 80, // Standard terminal size
            ..Default::default()
        })
        .expect("Unable to open pty");

    // TODO: There's no way to handle stdin
    let pty_reader = pair
        .master
        .try_clone_reader()
        .expect("Unable to clone pty reader");

    let (log_reader, log_writer) = std::io::pipe().expect("Failed to create log pipe");
    let pipes = (pair.slave, log_writer);
    (pty_reader, log_reader, pipes)
}

pub fn flush_pty(logger: &mut PtyOut) {
    let Some((pty, file)) = logger else {
        return;
    };
    // Not sure if flush actually working
    let _ = pty.flush();
    std::thread::sleep(Duration::from_millis(10));
    let _ = file.flush();
}

pub fn spawn_to_pipe(command: &mut Command, stdout_pipe: &PtyOut) -> Result<Child> {
    match stdout_pipe {
        Some(stdout) => stdout.0.spawn_command(command.into()),
        None => Ok(command.spawn().map_err(wrap_io_err!("Spawning"))?),
    }
}

pub fn write_to_pty(pty: &PtyOut, text: &str) {
    log_to_pty!(pty, "{}", text);
}

//
// based on portable-pty crate
// copied here since it isn't flexible enough
//

#[derive(Default)]
pub struct UnixPtySystem {}

/// Represents the size of the visible display area in the pty
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PtySize {
    /// The number of lines of text
    pub rows: u16,
    /// The number of columns of text
    pub cols: u16,
    /// The width of a cell in pixels.  Note that some systems never
    /// fill this value and ignore it.
    pub pixel_width: u16,
    /// The height of a cell in pixels.  Note that some systems never
    /// fill this value and ignore it.
    pub pixel_height: u16,
}

impl Default for PtySize {
    fn default() -> Self {
        PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

fn openpty(size: PtySize) -> Result<(UnixMasterPty, UnixSlavePty)> {
    let mut master: RawFd = -1;
    let mut slave: RawFd = -1;

    let mut size = winsize {
        ws_row: size.rows,
        ws_col: size.cols,
        ws_xpixel: size.pixel_width,
        ws_ypixel: size.pixel_height,
    };

    let result = unsafe {
        // BSDish systems may require mut pointers to some args
        #[allow(clippy::unnecessary_mut_passed)]
        libc::openpty(
            &mut master,
            &mut slave,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut size,
        )
    };

    if result != 0 {
        return Err(Error::from_last_io_error("Opening openpty"));
    }

    let master = UnixMasterPty {
        fd: PtyFd(unsafe { File::from_raw_fd(master) }),
    };
    let slave = UnixSlavePty {
        fd: PtyFd(unsafe { File::from_raw_fd(slave) }),
    };

    // Ensure that these descriptors will get closed when we execute
    // the child process.  This is done after constructing the Pty
    // instances so that we ensure that the Ptys get drop()'d if
    // the cloexec() functions fail (unlikely!).
    cloexec(master.fd.as_raw_fd())?;
    cloexec(slave.fd.as_raw_fd())?;

    Ok((master, slave))
}

pub struct PtyPair {
    // slave is listed first so that it is dropped first.
    // The drop order is stable and specified by rust rfc 1857
    pub slave: UnixSlavePty,
    pub master: UnixMasterPty,
}

impl UnixPtySystem {
    fn openpty(&self, size: PtySize) -> Result<PtyPair> {
        let (master, slave) = openpty(size)?;
        Ok(PtyPair {
            master: master,
            slave: slave,
        })
    }
}

struct PtyFd(pub File);
impl std::ops::Deref for PtyFd {
    type Target = File;
    fn deref(&self) -> &File {
        &self.0
    }
}

impl Read for PtyFd {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.0.read(buf) {
            Err(ref e) if e.raw_os_error() == Some(libc::EIO) => {
                // EIO indicates that the slave pty has been closed.
                // Treat this as EOF so that std::io::Read::read_to_string
                // and similar functions gracefully terminate when they
                // encounter this condition
                Ok(0)
            }
            x => x,
        }
    }
}

impl PtyFd {
    fn resize(&self, size: PtySize) -> Result<()> {
        let ws_size = winsize {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: size.pixel_width,
            ws_ypixel: size.pixel_height,
        };

        if unsafe {
            libc::ioctl(
                self.0.as_raw_fd(),
                libc::TIOCSWINSZ as _,
                &ws_size as *const _,
            )
        } != 0
        {
            return Err(Error::from_last_io_error("ioctl resize (TIOCSWINSZ)"));
        }

        Ok(())
    }

    fn get_size(&self) -> Result<PtySize> {
        let mut size: winsize = unsafe { mem::zeroed() };
        if unsafe {
            libc::ioctl(
                self.0.as_raw_fd(),
                libc::TIOCGWINSZ as _,
                &mut size as *mut _,
            )
        } != 0
        {
            return Err(Error::from_last_io_error("ioctl get size (TIOCGWINSZ)"));
        }
        Ok(PtySize {
            rows: size.ws_row,
            cols: size.ws_col,
            pixel_width: size.ws_xpixel,
            pixel_height: size.ws_ypixel,
        })
    }

    fn spawn_command(&self, cmd: &mut Command) -> Result<std::process::Child> {
        unsafe {
            cmd
                // .stdin(self.as_stdio()?)
                .stdout(self.try_clone().map_err(wrap_io_err!("Cloning pty"))?)
                .stderr(self.try_clone().map_err(wrap_io_err!("Cloning pty"))?)
                .pre_exec(move || {
                    // Clean up a few things before we exec the program
                    // Clear out any potentially problematic signal
                    // dispositions that we might have inherited
                    for signo in &[
                        libc::SIGCHLD,
                        libc::SIGHUP,
                        libc::SIGINT,
                        libc::SIGQUIT,
                        libc::SIGTERM,
                        libc::SIGALRM,
                    ] {
                        libc::signal(*signo, libc::SIG_DFL);
                    }

                    let empty_set: libc::sigset_t = std::mem::zeroed();
                    libc::sigprocmask(libc::SIG_SETMASK, &empty_set, std::ptr::null_mut());

                    // Establish ourselves as a session leader.
                    if libc::setsid() == -1 {
                        return Err(io::Error::last_os_error());
                    }

                    Ok(())
                })
        };

        let mut child = cmd.spawn().map_err(wrap_io_err!("Spawning cmd"))?;

        // Ensure that we close out the slave fds that Child retains;
        // they are not what we need (we need the master side to reference
        // them) and won't work in the usual way anyway.
        // In practice these are None, but it seems best to be move them
        // out in case the behavior of Command changes in the future.
        // child.stdin.take();
        child.stdout.take();
        child.stderr.take();

        Ok(child)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.flush().map_err(wrap_io_err!("Flushing pty"))
    }
}

/// Represents the master end of a pty.
/// The file descriptor will be closed when the Pty is dropped.
pub struct UnixMasterPty {
    fd: PtyFd,
}

/// Represents the slave end of a pty.
/// The file descriptor will be closed when the Pty is dropped.
pub struct UnixSlavePty {
    fd: PtyFd,
}

/// Helper function to set the close-on-exec flag for a raw descriptor
fn cloexec(fd: RawFd) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags == -1 {
        return Err(Error::from_last_io_error("fcntl to read flags"));
    }
    let result = unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) };
    if result == -1 {
        return Err(Error::from_last_io_error("fcntl to set CLOEXEC"));
    }
    Ok(())
}

impl UnixSlavePty {
    fn spawn_command(&self, builder: &mut Command) -> Result<std::process::Child> {
        Ok(self.fd.spawn_command(builder)?)
    }
    fn flush(&mut self) -> Result<()> {
        self.fd.flush()
    }
}

impl UnixMasterPty {
    #[allow(unused)]
    fn resize(&self, size: PtySize) -> Result<()> {
        self.fd.resize(size)
    }

    #[allow(unused)]
    fn get_size(&self) -> Result<PtySize> {
        self.fd.get_size()
    }

    fn try_clone_reader(&self) -> Result<Box<dyn Read + Send>> {
        let fd = PtyFd(
            self.fd
                .try_clone()
                .map_err(wrap_io_err!("Cloning pty fd"))?,
        );
        Ok(Box::new(fd))
    }
}
