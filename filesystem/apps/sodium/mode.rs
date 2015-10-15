/// A mode
#[derive(Clone, PartialEq, Copy, Hash)]
pub enum Mode {
    /// A primitive mode (no repeat, no delimiters, no preprocessing)
    Primitive(PrimitiveMode),
    /// Command mode
    Command(CommandMode),
}

#[derive(Clone, PartialEq, Copy, Hash)]
/// A command mode
pub enum CommandMode {
//    Visual(VisualOptions),
    /// Normal mode
    Normal,
}

#[derive(Clone, PartialEq, Copy, Hash)]
/// A primitive mode
pub enum PrimitiveMode {
    /// Insert mode
    Insert(InsertOptions),
}
