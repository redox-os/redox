use super::*;

#[derive(Clone, PartialEq, Copy)]
/// A mode
pub enum Mode {
    /// A primitive mode (no repeat, no delimiters, no preprocessing)
    Primitive(PrimitiveMode),
    /// Command mode
    Command(CommandMode),
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
