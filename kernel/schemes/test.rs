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
            ($ cond : expr) => (
                if ! string.is_empty() {
                    string.push('\n');
                }
                if $ cond {
                    string.push_str(&format!(concat!("\x1B[32mSUCCESS: ", stringify!($ cond), "\x1B[0m")));
                } else {
                    string.push_str(&format!(concat!("\x1B[31mFAILURE: ", stringify!($ cond), "\x1B[0m")));
                }
            );
            ($ cond : expr , $ ($ arg : tt) +) => (
                if ! string.is_empty() {
                    string.push('\n');
                }
                if $ cond {
                    string.push_str(&format!(concat!("\x1B[32mSUCCESS: ", stringify!($ cond), "\x1B[0m")));
                } else {
                    string.push_str(&format!(concat!("\x1B[31mFAILURE: ", stringify!($ cond), "\x1B[0m")));
                }
            );
        }

        {
            test!(true == true);
            test!(true == false);
        }

        Ok(box VecResource::new(Url::from_str("test:"), string.into_bytes()))
    }
}
