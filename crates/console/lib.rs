#![crate_name="console"]
#![crate_type="lib"]
#![feature(alloc)]
#![feature(collections)]
#![no_std]

extern crate alloc;

#[macro_use]
extern crate collections;

use alloc::boxed::Box;

use collections::String;
use collections::Vec;

use core::cmp;

pub use block::Block;
pub use color::Color;
pub use style::Style;

pub mod block;
pub mod color;
pub mod style;

pub struct Console {
    pub display: Box<[Block]>,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub foreground: Color,
    pub background: Color,
    pub style: Style,
    pub cursor: bool,
    pub redraw: bool,
    pub escape: bool,
    pub escape_sequence: bool,
    pub escape_extra: bool,
    pub sequence: Vec<String>,
    pub raw_mode: bool,
}

impl Console {
    pub fn new(w: usize, h: usize) -> Console {
        Console {
            display: vec![Block::new(); w * h].into_boxed_slice(),
            x: 0,
            y: 0,
            w: w,
            h: h,
            foreground: Color::ansi(7),
            background: Color::ansi(0),
            style: Style::Normal,
            cursor: true,
            redraw: true,
            escape: false,
            escape_sequence: false,
            escape_extra: false,
            sequence: Vec::new(),
            raw_mode: false,
        }
    }

    fn block(&self, c: char) -> Block {
        Block {
            c: c,
            fg: self.foreground,
            bg: self.background,
            style: self.style
        }
    }

    pub fn code(&mut self, c: char) {
        if self.escape_sequence {
            match c {
                '0' ... '9' => {
                    // Add a number to the sequence list
                    if let Some(mut value) = self.sequence.last_mut() {
                        value.push(c);
                    }
                },
                ';' => {
                    // Split sequence into list
                    self.sequence.push(String::new());
                },
                'm' => {
                    // Display attributes
                    let mut value_iter = self.sequence.iter();
                    while let Some(value_str) = value_iter.next() {
                        let value = value_str.parse::<u8>().unwrap_or(0);
                        match value {
                            0 => {
                                self.foreground = Color::ansi(7);
                                self.background = Color::ansi(0);
                                self.style = Style::Normal;
                            },
                            1 => {
                                self.style = Style::Bold;
                            },
                            30 ... 37 => self.foreground = Color::ansi(value - 30),
                            38 => match value_iter.next().map_or("", |s| &s).parse::<usize>().unwrap_or(0) {
                                2 => {
                                    //True color
                                    let r = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    let g = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    let b = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    self.foreground = Color::new(r, g, b);
                                },
                                5 => {
                                    //256 color
                                    let color_value = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    self.foreground = Color::ansi(color_value);
                                },
                                _ => {}
                            },
                            40 ... 47 => self.background = Color::ansi(value - 40),
                            48 => match value_iter.next().map_or("", |s| &s).parse::<usize>().unwrap_or(0) {
                                2 => {
                                    //True color
                                    let r = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    let g = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    let b = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    self.background = Color::new(r, g, b);
                                },
                                5 => {
                                    //256 color
                                    let color_value = value_iter.next().map_or("", |s| &s).parse::<u8>().unwrap_or(0);
                                    self.background = Color::ansi(color_value);
                                },
                                _ => {}
                            },
                            _ => {},
                        }
                    }

                    self.escape_sequence = false;
                },
                'A' => {
                    self.y -= cmp::min(self.y, self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(1));
                    self.escape_sequence = false;
                },
                'B' => {
                    self.y += cmp::min(self.h - 1 - self.y, self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(1));
                    self.escape_sequence = false;
                },
                'C' => {
                    self.x += cmp::min(self.w - 1 - self.x, self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(1));
                    self.escape_sequence = false;
                },
                'D' => {
                    self.x -= cmp::min(self.x, self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(1));
                    self.escape_sequence = false;
                },
                'H' | 'f' => {
                    let row = self.sequence.get(0).map_or("", |p| &p).parse::<isize>().unwrap_or(1);
                    self.y = cmp::max(0, row - 1) as usize;

                    let col = self.sequence.get(1).map_or("", |p| &p).parse::<isize>().unwrap_or(1);
                    self.x = cmp::max(0, col - 1) as usize;

                    self.escape_sequence = false;
                },
                'J' => {
                    match self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0) {
                        0 => {
                            let block = self.block(' ');
                            for c in self.display[self.y * self.w + self.x ..].iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        1 => {
                            let block = self.block(' ');
                            /* Should this add one? */
                            for c in self.display[.. self.y * self.w + self.x + 1].iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        2 => {
                            // Erase all
                            self.x = 0;
                            self.y = 0;
                            let block = self.block(' ');
                            for c in self.display.iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        _ => {}
                    }

                    self.escape_sequence = false;
                },
                'K' => {
                    match self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0) {
                        0 => {
                            let block = self.block(' ');
                            for c in self.display[self.y * self.w + self.x .. self.y * self.w + self.w].iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        1 => {
                            let block = self.block(' ');
                            /* Should this add one? */
                            for c in self.display[self.y * self.w .. self.y * self.w + self.x + 1].iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        2 => {
                            // Erase all
                            self.x = 0;
                            self.y = 0;
                            let block = self.block(' ');
                            for c in self.display.iter_mut() {
                                *c = block;
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        _ => {}
                    }

                    self.escape_sequence = false;
                },
                /*
                @MANSTART{terminal-raw-mode}
                INTRODUCTION
                    Since Redox has no ioctl syscall, it uses escape codes for switching to raw mode.

                ENTERING AND EXITING RAW MODE
                    Entering raw mode is done using CSI-r (^[r). Unsetting raw mode is done by CSI-R (^[R).

                RAW MODE
                    Raw mode means that the stdin must be handled solely by the program itself. It will not automatically be printed nor will it be modified in any way (modulo escape codes).

                    This means that:
                        - stdin is not printed.
                        - newlines are interpreted as carriage returns in stdin.
                        - stdin is not buffered, meaning that the stream of bytes goes directly to the program, without the user having to press enter.
                @MANEND
                */
                'r' => {
                    self.raw_mode = true;
                    self.escape_sequence = false;
                },
                'R' => {
                    self.raw_mode = false;
                    self.escape_sequence = false;
                },
                '?' => self.escape_extra = true,
                'h' if self.escape_extra => {
                    match self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0) {
                        25 => self.cursor = true,
                        _ => ()
                    }

                    self.escape_sequence = false;
                },
                'l' if self.escape_extra => {
                    match self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0) {
                        25 => self.cursor = false,
                        _ => ()
                    }

                    self.escape_sequence = false;
                },
                _ => self.escape_sequence = false,
            }

            if !self.escape_sequence {
                self.sequence.clear();
                self.escape = false;
                self.escape_extra = false;
            }
        } else {
            match c {
                '[' => {
                    // Control sequence initiator

                    self.escape_sequence = true;
                    self.sequence.push(String::new());
                },
                'c' => {
                    // Reset
                    self.x = 0;
                    self.y = 0;
                    self.raw_mode = false;
                    self.foreground = Color::ansi(7);
                    self.background = Color::ansi(0);
                    self.style = Style::Normal;
                    let block = self.block(' ');
                    for c in self.display.iter_mut() {
                        *c = block;
                    }
                    self.redraw = true;

                    self.escape = false;
                }
                _ => self.escape = false,
            }
        }
    }

    pub fn character(&mut self, c: char) {
        match c {
            '\0' => {},
            '\x1B' => self.escape = true,
            '\n' => {
                self.x = 0;
                self.y += 1;
                if ! self.raw_mode {
                    self.redraw = true;
                }
            },
            '\t' => self.x = ((self.x / 8) + 1) * 8,
            '\r' => self.x = 0,
            '\x08' => {
                if self.x >= 1 {
                    self.x -= 1;

                    if ! self.raw_mode {
                        self.display[self.y * self.w + self.x] = self.block(' ');
                    }
                }
            },
            ' ' => {
                self.display[self.y * self.w + self.x] = self.block(' ');

                self.x += 1;
            },
            _ => {
                self.display[self.y * self.w + self.x] = self.block(c);

                self.x += 1;
            }
        }

        if self.x >= self.w {
            self.x = 0;
            self.y += 1;
        }

        while self.y + 1 > self.h {
            for y in 1..self.h {
                for x in 0..self.w {
                    let c = self.display[y * self.w + x];
                    self.display[(y - 1) * self.w + x] = c;
                }
            }
            let block = self.block(' ');
            for x in 0..self.w {
                self.display[(self.h - 1) * self.w + x] = block;
            }
            self.y -= 1;
        }
    }

    pub fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            let c = *byte as char;

            if self.escape {
                self.code(c);
            } else {
                self.character(c);
            }
        }
    }
}
