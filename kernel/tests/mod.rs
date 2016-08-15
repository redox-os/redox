use arch::interrupt::{enable_interrupts, halt};

use syscall::{self, Error};

/// Test halting
#[test]
fn halt_with_interrupts() {
    unsafe {
        //enable_interrupts();
        //halt();
    }
}

/// Test stdio
#[test]
fn stdio() {
    // Test opening stdin
    assert_eq!(syscall::open(b"debug:", 0), Ok(0));

    // Test opening stdout
    assert_eq!(syscall::open(b"debug:", 0), Ok(1));

    // Test opening stderr
    assert_eq!(syscall::open(b"debug:", 0), Ok(2));

    // Test writing stdout
    let stdout_str = b"STDOUT";
    assert_eq!(syscall::write(1, stdout_str), Ok(stdout_str.len()));

    // Test writing stderr
    let stderr_str = b"STDERR";
    assert_eq!(syscall::write(2, stderr_str), Ok(stderr_str.len()));
}

/// Test that invalid reads/writes cause errors
#[test]
fn invalid_path() {
    assert_eq!(syscall::read(999, &mut []), Err(Error::BadFile));
    assert_eq!(syscall::write(999, &[]), Err(Error::BadFile));
}
