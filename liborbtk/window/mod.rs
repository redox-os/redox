#[cfg(target_os = "redox")]
extern crate orbital;

#[cfg(target_os = "redox")]
use self::orbital::Window as InnerWindow;

use super::Widget;

pub struct Window {
    inner: InnerWindow,
    widgets: Vec<Box<Widget>>
}

impl Widget for Window {}
