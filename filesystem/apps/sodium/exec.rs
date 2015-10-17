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
                'h' => self.left(n as usize),
                'j' => self.down(n as usize),
                'k' => self.up(n as usize),
                'l' => self.right(n as usize),
                'x' => self.delete(),
                ' ' => self.next(),
                _ => {},
            },
            Primitive(Insert(_)) => {
                self.insert(cmd);
            },
        }
    }
}
