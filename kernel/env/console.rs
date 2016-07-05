extern crate console;

use alloc::boxed::Box;

use collections::String;

use common::debug::SerialConsole;
use common::event::{self, Event, EventOption};

use core::mem;

use graphics::color::Color;
use graphics::display::Display;

use sync::WaitQueue;

pub struct Console {
    pub display: Option<Box<Display>>,
    pub inner: Option<console::Console>,
    pub draw: bool,
    pub command: String,
    pub commands: WaitQueue<String>
}

impl Console {
    pub fn new() -> Console {
        let display_option = Display::root();
        let inner_option = if let Some(ref display) = display_option {
            Some(console::Console::new(display.width/8, display.height/16))
        } else {
            None
        };
        Console {
            display: display_option,
            inner: inner_option,
            draw: false,
            command: String::new(),
            commands: WaitQueue::new()
        }
    }

    pub fn event(&mut self, event: Event) {
        match event.to_option() {
            EventOption::Key(key_event) => {
                if key_event.pressed {
                    let raw_mode = if let Some(ref inner) = self.inner {
                        inner.raw_mode
                    } else {
                        false
                    };

                    if raw_mode {
                        match key_event.scancode {
                            event::K_BKSP => self.command.push_str("\x7F"),
                            event::K_UP => self.command.push_str("\x1B[A"),
                            event::K_DOWN => self.command.push_str("\x1B[B"),
                            event::K_RIGHT => self.command.push_str("\x1B[C"),
                            event::K_LEFT => self.command.push_str("\x1B[D"),
                            _ => match key_event.character {
                                '\0' => {},
                                c => {
                                    self.command.push(c);
                                }
                            },
                        }

                        if ! self.command.is_empty() {
                            let mut command = String::new();
                            mem::swap(&mut self.command, &mut command);
                            self.commands.send(command, "Console::event command (raw)");
                        }
                    } else {
                        match key_event.scancode {
                            event::K_BKSP => if ! self.command.is_empty() {
                                if let Some(ref mut inner) = self.inner {
                                    inner.redraw = true;
                                }

                                self.write(&[8]);
                                self.command.pop();
                            },
                            _ => match key_event.character {
                                '\0' => (),
                                c => {
                                    if let Some(ref mut inner) = self.inner {
                                        inner.redraw = true;
                                    }

                                    self.write(&[c as u8]);
                                    self.command.push(c);

                                    if c == '\n' {
                                        let mut command = String::new();
                                        mem::swap(&mut self.command, &mut command);
                                        self.commands.send(command, "Console::event command (not raw)");
                                    }
                                }
                            },
                        }
                    }
                }
            }
            _ => (),
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        if self.draw && self.inner.is_some() {
            if let Some(ref mut inner) = self.inner {
                inner.write(bytes);
                if inner.redraw {
                    inner.redraw = false;
                    if let Some(ref mut display) = self.display {
                        display.set(Color {
                            data: inner.background.data
                        });
                        for y in 0..inner.h {
                            for x in 0..inner.w {
                                let block = inner.display[y * inner.w + x];
                                display.rect(x * 8, y * 16, 8, 16, Color {
                                    data: block.bg.data
                                });
                                if block.c != ' ' {
                                    display.char(x * 8, y * 16, block.c, Color {
                                        data: block.fg.data
                                    });
                                }
                            }
                        }
                        if inner.cursor {
                            display.rect(inner.x * 8, inner.y * 16, 8, 16, Color {
                                data: inner.foreground.data
                            });
                        }
                        display.flip();
                    }
                }
            }
        } else {
            SerialConsole::new().write(bytes);
        }
    }
}
