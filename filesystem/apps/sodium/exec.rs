use super::*;
use redox::*;

impl Editor {
    /// Execute a instruction
    pub fn exec(&mut self, Inst(para, cmd): Inst) {
        use super::Key::*;
        use super::Mode::*;
        use super::PrimitiveMode::*;
        use super::CommandMode::*;

        let n = para.d();
        match cmd {
            Ctrl(b) => self.key_state.ctrl = b,
            Alt(b) => self.key_state.alt = b,
            Shift(b) => self.key_state.shift = b,
            _ => {},
        }

        if cmd == Char(' ') && self.key_state.shift {

            let mode = self.cursor().mode;

            match mode {
                Primitive(Prompt) => self.prompt = String::new(),
                _ => {},
            }
            self.cursor_mut().mode = Mode::Command(CommandMode::Normal);

        } else if self.key_state.alt && cmd == Key::Char(' ') {

            self.next_cursor();

        } else if self.key_state.alt {

            let new_pos = self.to_motion(Inst(para, cmd));
            self.goto(new_pos);

        } else {
            match self.cursor().mode {
                Command(Normal) => match cmd {
                    Char('i') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                            InsertOptions {
                                mode: InsertMode::Insert,
                            }));

                    },
                    Char('o') => {
                        // TODO: Autoindent (keep the same indentation level)
                        let y = self.y();
                        let ind = if self.options.autoindent {
                            self.get_indent(y)
                        } else {
                            VecDeque::new()
                        };
                        let last = ind.len();
                        self.text.insert(y + 1, ind);
                        self.goto((last, y + 1));
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                            InsertOptions {
                                mode: InsertMode::Insert,
                            }));
                    }
                    Char('h') => self.goto_left(n),
                    Char('j') => self.goto_down(n),
                    Char('k') => self.goto_up(n),
                    Char('l') => self.goto_right(n),
                    Char('J') => self.goto_down(15 * n),
                    Char('K') => self.goto_up(15 * n),
                    Char('x') => self.delete(),
                    Char('X') => {
                        self.goto_previous();
                        self.delete();
                    },
                    Char('L') => self.goto_ln_end(),
                    Char('H') => self.cursor_mut().x = 0,
                    Char('r') => {
                        let x = self.x();
                        let y = self.y();
                        self.text[y][x] = self.next_char();
                    },
                    Char('R') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                            InsertOptions {
                                mode: InsertMode::Replace,
                            }));
                    },
                    Char('d') => {
                        let ins = self.next_inst();
                        let motion = self.to_motion(ins);
                        self.remove_rb(motion);
                    },
                    Char('G') => {
                        let last = self.text.len() - 1;
                        self.goto((0, last));
                    },
                    Char('g') => {
                        if let Parameter::Int(n) = para {
                            self.goto((0, n - 1));
                        } else {
                            let inst = self.next_inst();
                            let new = self.to_motion(inst);
                            self.cursor_mut().x = new.0;
                            self.cursor_mut().y = new.1;
                        }

                    },
                    Char('b') => {
                        // Branch cursor
                        let cursor = self.cursor().clone();
                        self.cursors.push(cursor);
                    },
                    Char('B') => {
                        // Delete cursor
                        self.cursors.remove(self.current_cursor as usize);
                        self.next_cursor();
                    },
                    Char(';') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Prompt);
                    },
//                    ????
//                    Char('K') => {
//                        self.goto((0, 0));
//                    },
//                    Char('J') => {
//                        self.goto((0, self.text.len() - 1));
//                    },
                    Char(' ') => self.goto_next(),
                    _ => {},
                },
                Primitive(Insert(opt)) => {
                    self.insert(cmd, opt);
                },
                Primitive(Prompt) => {
                    match cmd {
                        Char('\n') => {
                            self.cursor_mut().mode = Command(Normal);
                            let cmd = self.prompt.clone();

                            self.invoke(cmd);
                            self.prompt = String::new();
                        },
                        Char(c) => self.prompt.push(c),
                        _ => {},
                    }
                },
            }
        }
    }
}
