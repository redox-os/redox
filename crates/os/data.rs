use file::File;
use dir::Dir;

pub enum DataType {
    File = 102,
    Dir  = 100,
    None = 0,
}

impl DataType {
    pub fn from_byte(b: u8) -> Self {
        match b {
            102 => DataType::File,
            100 => DataType::Dir,
            _   => DataType::None,
        }
    }
}

pub enum Data<'a> {
    File(File<'a>),
    Dir(Dir<'a>),
}

impl<'a> Data<'a> {
    pub fn name(&self) -> &[u8] {
        match self {
            &File(ref f) => f.name,
            &Dir(ref d) => d.name,
        }
    }
}
