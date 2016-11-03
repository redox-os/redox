extern crate ransid;

use std::collections::{BTreeSet, VecDeque};

use orbclient::{Event, EventOption};
use syscall::error::*;

use display::Display;
use screen::Screen;

pub struct TextScreen {
    pub console: ransid::Console,
    pub display: Display,
    pub changed: BTreeSet<usize>,
    pub ctrl: bool,
    pub input: VecDeque<u8>,
    pub end_of_input: bool,
    pub cooked: VecDeque<u8>,
    pub requested: usize
}

impl TextScreen {
    pub fn new(display: Display) -> TextScreen {
        TextScreen {
            console: ransid::Console::new(display.width/8, display.height/16),
            display: display,
            changed: BTreeSet::new(),
            ctrl: false,
            input: VecDeque::new(),
            end_of_input: false,
            cooked: VecDeque::new(),
            requested: 0
        }
    }
}

impl Screen for TextScreen {
    fn width(&self) -> usize {
        self.console.w
    }

    fn height(&self) -> usize {
        self.console.h
    }

    fn event(&mut self, flags: usize) -> Result<usize> {
        self.requested = flags;
        Ok(0)
    }

    fn map(&self, offset: usize, size: usize) -> Result<usize> {
        Err(Error::new(EBADF))
    }

    fn input(&mut self, event: &Event) {
        let mut buf = vec![];

        match event.to_option() {
            EventOption::Key(key_event) => {
                if key_event.scancode == 0x1D {
                    self.ctrl = key_event.pressed;
                } else if key_event.pressed {
                    match key_event.scancode {
                        0x47 => { // Home
                            buf.extend_from_slice(b"\x1B[H");
                        },
                        0x48 => { // Up
                            buf.extend_from_slice(b"\x1B[A");
                        },
                        0x49 => { // Page up
                            buf.extend_from_slice(b"\x1B[5~");
                        },
                        0x4B => { // Left
                            buf.extend_from_slice(b"\x1B[D");
                        },
                        0x4D => { // Right
                            buf.extend_from_slice(b"\x1B[C");
                        },
                        0x4F => { // End
                            buf.extend_from_slice(b"\x1B[F");
                        },
                        0x50 => { // Down
                            buf.extend_from_slice(b"\x1B[B");
                        },
                        0x51 => { // Page down
                            buf.extend_from_slice(b"\x1B[6~");
                        },
                        0x52 => { // Insert
                            buf.extend_from_slice(b"\x1B[2~");
                        },
                        0x53 => { // Delete
                            buf.extend_from_slice(b"\x1B[3~");
                        },
                        _ => {
                            let c = match key_event.character {
                                c @ 'A' ... 'Z' if self.ctrl => ((c as u8 - b'A') + b'\x01') as char,
                                c @ 'a' ... 'z' if self.ctrl => ((c as u8 - b'a') + b'\x01') as char,
                                c => c
                            };

                            if c != '\0' {
                                buf.extend_from_slice(&[c as u8]);
                            }
                        }
                    }
                }
            },
            _ => () //TODO: Mouse in terminal
        }

        if self.console.raw_mode {
            for &b in buf.iter() {
                self.input.push_back(b);
            }
        } else {
            for &b in buf.iter() {
                match b {
                    b'\x03' => {
                        self.end_of_input = true;
                        let _ = self.write(b"^C\n", true);
                    },
                    b'\x08' | b'\x7F' => {
                        if let Some(_c) = self.cooked.pop_back() {
                            let _ = self.write(b"\x08", true);
                        }
                    },
                    b'\n' | b'\r' => {
                        self.cooked.push_back(b);
                        while let Some(c) = self.cooked.pop_front() {
                            self.input.push_back(c);
                        }
                        let _ = self.write(b"\n", true);
                    },
                    _ => {
                        self.cooked.push_back(b);
                        let _ = self.write(&[b], true);
                    }
                }
            }
        }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut i = 0;

        while i < buf.len() && ! self.input.is_empty() {
            buf[i] = self.input.pop_front().unwrap();
            i += 1;
        }

        if i == 0 {
            self.end_of_input = false;
        }

        Ok(i)
    }

    fn will_block(&self) -> bool {
        self.input.is_empty() && ! self.end_of_input
    }

    fn write(&mut self, buf: &[u8], sync: bool) -> Result<usize> {
        if self.console.cursor && self.console.x < self.console.w && self.console.y < self.console.h {
            let x = self.console.x;
            let y = self.console.y;
            let color = self.console.background;
            self.display.rect(x * 8, y * 16, 8, 16, color.data);
            self.changed.insert(y);
        }

        {
            let display = &mut self.display;
            let changed = &mut self.changed;
            self.console.write(buf, |event| {
                match event {
                    ransid::Event::Char { x, y, c, color, bold, .. } => {
                        display.char(x * 8, y * 16, c, color.data, bold, false);
                        changed.insert(y);
                    },
                    ransid::Event::Rect { x, y, w, h, color } => {
                        display.rect(x * 8, y * 16, w * 8, h * 16, color.data);
                        for y2 in y..y + h {
                            changed.insert(y2);
                        }
                    },
                    ransid::Event::Scroll { rows, color } => {
                        display.scroll(rows * 16, color.data);
                        for y in 0..display.height/16 {
                            changed.insert(y);
                        }
                    }
                }
            });
        }

        if self.console.cursor && self.console.x < self.console.w && self.console.y < self.console.h {
            let x = self.console.x;
            let y = self.console.y;
            let color = self.console.foreground;
            self.display.rect(x * 8, y * 16, 8, 16, color.data);
            self.changed.insert(y);
        }

        if ! self.console.raw_mode && sync {
            self.sync();
        }

        Ok(buf.len())
    }

    fn seek(&mut self, _pos: usize, _whence: usize) -> Result<usize> {
        Ok(0)
    }

    fn sync(&mut self) {
        let width = self.display.width;
        for change in self.changed.iter() {
            self.display.sync(0, change * 16, width, 16);
        }
        self.changed.clear();
    }

    fn redraw(&mut self) {
        let width = self.display.width;
        let height = self.display.height;
        self.display.sync(0, 0, width, height);
        self.changed.clear();
    }
}
