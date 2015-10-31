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

        if cmd.key == Key::Char(' ') && cmd.shift {

            let mode = self.cursor().mode;

            match mode {
                Primitive(Prompt) => self.prompt = String::new(),
                _ => {},
            }
            self.cursor_mut().mode = Mode::Command(CommandMode::Normal);

        } else if cmd.alt && cmd.key == Key::Char(' ') {

            self.next_cursor();

        } else if cmd.alt {

            if let Some(m) = self.to_motion(Inst(para, cmd)) {
                self.goto(m);
            }

        } else {
            match self.cursor().mode {
                Command(Normal) => match cmd.key {
                    Char('i') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                            InsertOptions {
                                mode: InsertMode::Insert,
                            }));

                    },
                    Char('a') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Insert(
                            InsertOptions {
                                mode: InsertMode::Append,
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
                    Char('h') => {
                        let left = self.left(n);
                        self.goto(left);
                    },
                    Char('j') => {
                        let down = self.down(n);
                        self.goto(down);
                    },
                    Char('k') => {
                        let up = self.up(n);
                        self.goto(up);
                    },
                    Char('l') => {
                        let right = self.right(n);
                        self.goto(right);
                    },
                    Char('J') => {
                        let down = self.down(15 * n);
                        self.goto(down);
                    },
                    Char('K') => {
                        let up = self.up(15 * n);
                        self.goto(up);
                    },
                    Char('x') => self.delete(),
                    Char('X') => {
                        let previous = self.previous();
                        if let Some(p) = previous {
                            self.goto(p);
                        }
                        self.delete();
                    },
                    Char('L') => {
                        let ln_end = self.ln_end();
                        self.goto(ln_end);
                    },
                    Char('H') => self.cursor_mut().x = 0,
                    Char('r') => {
                        let (x, y) = self.pos();
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
                        if let Some(m) = self.to_motion_unbounded(ins) {
                            self.remove_rb(m);
                        }
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
                            if let Some(m) = self.to_motion(inst) {
                                self.cursor_mut().x = m.0;
                                self.cursor_mut().y = m.1;
                            }
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
                    Char('t') => {
                        let ch = self.next_char();

                        let pos = self.next_ocur(ch, n);
                        if let Some(p) = pos {
                            self.goto(p);
                        }
                    },
                    Char(';') => {
                        self.cursor_mut().mode = Mode::Primitive(PrimitiveMode::Prompt);
                    },
//
//                    ????
//                    Char('K') => {
//                        self.goto((0, 0));
//                    },
//                    Char('J') => {
//                        self.goto((0, self.text.len() - 1));
//                    },
                    Char(' ') => {
                        let next = self.next();
                        if let Some(p) = next {
                            self.goto(p);
                        }
                    },
                    Char(c) => {
                        self.status_bar.msg = format!("Unknown command: {}", c);
                        self.redraw_status_bar();
                    }
                    _ => {
                        self.status_bar.msg = format!("Unknown command");
                        self.redraw_status_bar();
                    },
                },
                Primitive(Insert(opt)) => {
                    self.insert(cmd.key, opt);
                },
                Primitive(Prompt) => {
                    match cmd.key {
                        Char('\n') => {
                            self.cursor_mut().mode = Command(Normal);
                            let cmd = self.prompt.clone();

                            self.invoke(cmd);
                            self.prompt = String::new();
                        },
                        Backspace => {
                            self.prompt.pop();
                        },
                        Char(c) => self.prompt.push(c),
                        _ => {},
                    }
                },
            }
        }
    }
}
