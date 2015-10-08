pub use redox::File::{ Read, Write };

pub mod prelude {
    pub use super::{ Read, Write }; // BufRead, Seek };
//    pub use fs::PathExt;
}
