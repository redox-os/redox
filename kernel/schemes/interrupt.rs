use alloc::boxed::Box;

use collections::string::String;

use schemes::{Result, KScheme, Resource, Url, VecResource};

pub struct InterruptScheme;

impl KScheme for InterruptScheme {
    fn scheme(&self) -> &str {
        "interrupt"
    }

    fn open(&mut self, _: &Url, _: usize) -> Result<Box<Resource>> {
        let mut string = format!("{:<6}{:<16}", "INT", "COUNT");

        {
            let interrupts = ::env().interrupts.lock();
            for interrupt in 0..interrupts.len() {
                let count = interrupts[interrupt];

                if count > 0 {
                    string = string + "\n" + &format!("{:<6X}{:<16}", interrupt, count);
                }
            }
        }

        Ok(box VecResource::new(Url::from_str("interrupt:"), string.into_bytes()))
    }
}
