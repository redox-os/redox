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
            test!(true, "Passing test with format {:X}", 0x12345678);
            test!(false, "Failing test with description");
        }

        {
            use common::get_slice::GetSlice;
            let array = [1, 2, 3, 4, 5];

            test!(array.get_slice(100..100) == &[]);
            test!(array.get_slice(..100) == &array);
            test!(array.get_slice(1..) == &array[1..]);
            test!(array.get_slice(1..2) == &[2]);
            test!(array.get_slice(3..5) == &[4, 5]);
            test!(array.get_slice(3..7) == &[4, 5]);
            test!(array.get_slice(3..4) == &[4]);
            test!(array.get_slice(4..2) == &[]);
            test!(array.get_slice(4..1) == &[]);
            test!(array.get_slice(20..) == &[]);
            test!(array.get_slice(..) == &array);
        }

        Ok(box VecResource::new(Url::from_str("test:"), string.into_bytes()))
    }
}
