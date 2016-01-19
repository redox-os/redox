use alloc::boxed::Box;

use collections::string::String;

use scheduler::context;

use schemes::{Result, KScheme, Resource, Url, VecResource};

pub struct TestScheme;

impl KScheme for TestScheme {
    fn scheme(&self) -> &str {
        "test"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        let mut string = String::new();

        macro_rules! test {
            ($cond:expr) => (
                if ! string.is_empty() {
                    string.push('\n');
                }
                if $ cond {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($cond));
                string.push_str("\x1B[0m");
            );
            ($cond:expr, $($arg:tt)*) => (
                if ! string.is_empty() {
                    string.push('\n');
                }
                if $ cond {
                    string.push_str("\x1B[32mSUCCESS: ");
                } else {
                    string.push_str("\x1B[31mFAILURE: ");
                }
                string.push_str(stringify!($cond));
                string.push_str(": ");
                string.push_str(&format!($($arg)*));
                string.push_str("\x1B[0m");
            );
        }

        {
            test!(true == true);
            test!(true == false);
            test!(false, "Failing test with description");
            test!(true, "Passing test with format {:X}", 0x12345678);
        }

        Ok(box VecResource::new(Url::from_str("test:"), string.into_bytes()))
    }
}
