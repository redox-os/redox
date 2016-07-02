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

use color::Color;

pub mod color;

fn ansi_color(value: u8) -> Color {
    match value {
        0 => Color::new(0x00, 0x00, 0x00),
        1 => Color::new(0x80, 0x00, 0x00),
        2 => Color::new(0x00, 0x80, 0x00),
        3 => Color::new(0x80, 0x80, 0x00),
        4 => Color::new(0x00, 0x00, 0x80),
        5 => Color::new(0x80, 0x00, 0x80),
        6 => Color::new(0x00, 0x80, 0x80),
        7 => Color::new(0xc0, 0xc0, 0xc0),
        8 => Color::new(0x80, 0x80, 0x80),
        9 => Color::new(0xff, 0x00, 0x00),
        10 => Color::new(0x00, 0xff, 0x00),
        11 => Color::new(0xff, 0xff, 0x00),
        12 => Color::new(0x00, 0x00, 0xff),
        13 => Color::new(0xff, 0x00, 0xff),
        14 => Color::new(0x00, 0xff, 0xff),
        15 => Color::new(0xff, 0xff, 0xff),
        16 ... 231 => {
            let convert = |value: u8| -> u8 {
                match value {
                    0 => 0,
                    _ => value * 0x28 + 0x28
                }
            };

            let r = convert((value - 16)/36 % 6);
            let g = convert((value - 16)/6 % 6);
            let b = convert((value - 16) % 6);
            Color::new(r, g, b)
        },
        232 ... 255 => {
            let gray = (value - 232) * 10 + 8;
            Color::new(gray, gray, gray)
        },
        _ => Color::new(0, 0, 0)
    }
}

pub struct Console {
    pub display: Box<[(char, Color)]>,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub foreground: Color,
    pub background: Color,
    pub draw: bool,
    pub redraw: bool,
    pub escape: bool,
    pub escape_sequence: bool,
    pub sequence: Vec<String>,
    pub raw_mode: bool,
}

impl Console {
    pub fn new(w: usize, h: usize) -> Console {
        Console {
            display: vec![('\0', ansi_color(0)); w * h].into_boxed_slice(),
            x: 0,
            y: 0,
            w: w,
            h: h,
            foreground: ansi_color(7),
            background: ansi_color(0),
            draw: false,
            redraw: true,
            escape: false,
            escape_sequence: false,
            sequence: Vec::new(),
            raw_mode: false,
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
                                self.foreground = ansi_color(7);
                                self.background = ansi_color(0);
                            },
                            30 ... 37 => self.foreground = ansi_color(value - 30),
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
                                    self.foreground = ansi_color(color_value);
                                },
                                _ => {}
                            },
                            40 ... 47 => self.background = ansi_color(value - 40),
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
                                    self.background = ansi_color(color_value);
                                },
                                _ => {}
                            },
                            _ => {},
                        }
                    }

                    self.escape_sequence = false;
                },
                'J' => {
                    match self.sequence.get(0).map_or("", |p| &p).parse::<usize>().unwrap_or(0) {
                        0 => {
                            //TODO: Erase down
                        },
                        1 => {
                            //TODO: Erase up
                        },
                        2 => {
                            // Erase all
                            self.x = 0;
                            self.y = 0;
                            for c in self.display.iter_mut() {
                                *c = ('\0', self.background);
                            }
                            if ! self.raw_mode {
                                self.redraw = true;
                            }
                        },
                        _ => {}
                    }

                    self.escape_sequence = false;
                },
                'H' | 'f' => {
                    self.display[self.y * self.w + self.x] = ('\0', self.background);

                    let row = self.sequence.get(0).map_or("", |p| &p).parse::<isize>().unwrap_or(1);
                    self.y = cmp::max(0, row - 1) as usize;

                    let col = self.sequence.get(1).map_or("", |p| &p).parse::<isize>().unwrap_or(1);
                    self.x = cmp::max(0, col - 1) as usize;

                    self.display[self.y * self.w + self.x] = ('\0', self.foreground);

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
                _ => self.escape_sequence = false,
            }

            if !self.escape_sequence {
                self.sequence.clear();
                self.escape = false;
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
                    self.foreground = ansi_color(7);
                    self.background = ansi_color(0);
                    for c in self.display.iter_mut() {
                        *c = ('\0', self.background);
                    }
                    self.redraw = true;

                    self.escape = false;
                }
                _ => self.escape = false,
            }
        }
    }

    pub fn character(&mut self, c: char) {
        self.display[self.y * self.w + self.x] = ('\0', self.background);

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
                }

                self.display[self.y * self.w + self.x] = ('\0', self.background);
            },
            _ => {
                self.display[self.y * self.w + self.x] = (c, self.foreground);

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
            for x in 0..self.w {
                self.display[(self.h - 1) * self.w + x] = ('\0', self.background);
            }
            self.y -= 1;
        }

        self.display[self.y * self.w + self.x] = ('\0', self.foreground);
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
