use core::ops::DerefMut;
use redox::ion::command::{Application};

pub fn main() {
    unsafe {
        let mut app = box Application::new();
        application = app.deref_mut();
        app.main();
    }
}
