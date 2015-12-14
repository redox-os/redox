#![crate_type="lib"]
#![feature(box_syntax)]

extern crate orbital;

pub use orbital::{Color, Event, EventOption, KeyEvent, MouseEvent, Point, Size};

pub use label::Label;
pub use progress_bar::ProgressBar;
pub use rect::Rect;
pub use renderer::Renderer;
pub use widget::Widget;
pub use window::Window;

pub mod label;
pub mod progress_bar;
pub mod rect;
pub mod renderer;
pub mod widget;
pub mod window;
