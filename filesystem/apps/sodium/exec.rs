use super::*;
use redox::*;

impl Editor {
    /// Execute a instruction
    pub fn exec(&mut self, Inst(repeat, cmd): Inst) {
        use super::Key::*;
        use super::Mode::*;
        use super::PrimitiveMode::*;
        use super::CommandMode::*;

        let n = repeat.d();
        match cmd {
            Ctrl(b) => self.key_state.ctrl = b,
            Alt(b) => self.key_state.alt = b,
            Shift(b) => self.key_state.shift = b,
            _ => {},
        }

        if cmd == Char(' ') && self.key_state.shift {
            self.cursor_mut().mode = Mode::Command(CommandMode::Normal);
        } else if self.key_state.alt {
            let new_pos = self.to_motion(Inst(repeat, cmd));
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
                    Char('h') => self.goto_left(n),
                    Char('j') => self.goto_down(n),
                    Char('k') => self.goto_up(n),
                    Char('l') => self.goto_right(n),
                    Char('J') => self.goto_down(15),
                    Char('K') => self.goto_up(15),
                    Char('x') => self.delete(),
                    Char('X') => {
                        self.goto_previous();
                        self.delete();
                    },
                    Char('L') => self.goto_ln_end(),
                    Char('H') => self.cursor_mut().x = 0,
                    Char('r') => {
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
                    Char('g') => {
                        let inst = self.next_inst();
                        let new = self.to_motion(inst);
                        self.cursor_mut().x = new.0;
                        self.cursor_mut().y = new.1;

                    },
                    Char(' ') => self.goto_next(),
                    _ => {},
                },
                Primitive(Insert(opt)) => {
                    self.insert(cmd, opt);
                },
            }
        }
    }
}
