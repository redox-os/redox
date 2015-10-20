use super::*;
use redox::*;

#[derive(Clone, PartialEq, Copy)]
/// A mode
pub enum Mode {
    /// A primitive mode (no repeat, no delimiters, no preprocessing)
    Primitive(PrimitiveMode),
    /// Command mode
    Command(CommandMode),
}

impl Mode {
    pub fn to_string(self) -> String {
        use self::Mode::*;
        use self::PrimitiveMode::*;
        use self::CommandMode::*;
        match self {
            Command(Normal) => "Normal",
            Primitive(Insert(_)) => "Insert",
        }.to_string()
    }
}

#[derive(Clone, PartialEq, Copy)]
/// A command mode
pub enum CommandMode {
//    Visual(VisualOptions),
    /// Normal mode
    Normal,
}

#[derive(Clone, PartialEq, Copy)]
/// A primitive mode
pub enum PrimitiveMode {
    /// Insert mode
    Insert(InsertOptions),
}
