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
                'J' => self.down(15),
                'K' => self.up(15),
                'x' => self.delete(),
                'X' => {
                    self.previous();
                    self.delete();
                },
                '$' => self.cursor_mut().x = self.text[self.y()].len(),
                '0' => self.cursor_mut().x = 0,
                'r' => {
                    loop {
                        if let EventOption::Key(k) = self.window.poll()
                                                     .unwrap_or(Event::new())
                                                     .to_option() {
                            if k.pressed {
                                let x = self.x();
                                let y = self.y();
                                self.text[y][x] = k.character;
                                break;
                            }
                        }
                    }
                },
                ' ' => self.next(),
                _ => {},
            },
            Primitive(Insert(_)) => {
                self.insert(cmd);
            },
        }
    }
}
