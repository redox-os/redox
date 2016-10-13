#![feature(question_mark)]

extern crate syscall;

pub use resource::Resource;
pub use scheme::ResourceScheme;

mod resource;
mod scheme;
