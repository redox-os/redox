use alloc::boxed::Box;

use collections::string::String;

use fs::{KScheme, Resource, Url, VecResource};

use system::error::Result;

#[macro_export]
macro_rules! test {
    ($test:expr) => (
        if !$test {
            return false;
        }
    )
}

#[macro_export]
macro_rules! succ {
    () => (
        return true;
    )
}

#[macro_export]
macro_rules! fail {
    () => (
        return false;
    )
}

// Add your test here!
pub mod get_slice;
pub mod meta;

pub struct TestScheme;

impl KScheme for TestScheme {
    fn scheme(&self) -> &str {
        "test"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        let mut string = String::new();

        macro_rules! reg_test {
            (! $test:path) => (
                if !$test() {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($test));
                string.push_str("\x1B[0m\n");
            );
            (! $test:path, $($arg:tt)*) => (
                if !$test() {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($test));
                string.push_str(": ");
                string.push_str(&format!($($arg)*));
                string.push_str("\x1B[0m\n");
            );
            ($test:path) => (
                if $test() {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($test));
                string.push_str("\x1B[0m\n");
            );
            ($test:path, $($arg:tt)*) => (
                if $test() {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($test));
                string.push_str(": ");
                string.push_str(&format!($($arg)*));
                string.push_str("\x1B[0m\n");
            );
        }

        // Add your test here!
        reg_test!(meta::meta_test_woah, "Testing the testing (wut)");
        reg_test!(!meta::meta_test_woah_fail, "Testing the fail testing (wut)");
        reg_test!(get_slice::test, "GetSlice");

        Ok(box VecResource::new("test:", string.into_bytes()))
    }
}
