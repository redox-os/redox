use super::*;
use redox::*;

impl Editor {
    pub fn exec(&mut self, Inst(n, cmd): Inst) {
        use super::Mode::*;
        use super::PrimitiveMode::*;
        use super::CommandMode::*;
        match self.cursor().mode {
            Command(Normal) => match cmd {
                'i' => {
                    self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                        InsertOptions {
                            mode: InsertMode::Insert,
                        }));

                },
                'h' => self.left(),
                'j' => self.down(),
                'k' => self.up(),
                'l' => self.right(),
                'x' => self.delete(),
                _ => {},
            },
            Primitive(Insert(_)) => {
                self.insert(cmd);
            },
        }
    }
}
