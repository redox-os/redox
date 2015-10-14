use redox::*;

// TODO: Structure using loops

use super::Mode;
use super::Mode::*;
use super::Editor;

pub fn exec(editor: &mut Editor, mode: &mut Mode, multiplier: &mut Option<u32>, last_change: &mut String, key_event: KeyEvent, window: &mut Window, swap: &mut usize, period: &mut String, is_recording: &mut bool) {
    match (*mode, key_event.scancode) {
        (Insert, K_ESC) => {
            *mode = Normal;
        },
        (Insert, K_BKSP) => editor.backspace(window),
        (Insert, K_DEL) => editor.delete(window),
        (_, K_F5) => editor.reload(window),
        (_, K_F6) => editor.save(window),
        (_, K_HOME) => editor.offset = 0,
        (_, K_UP) => editor.up(),
        (_, K_LEFT) => editor.left(),
        (_, K_RIGHT) => editor.right(),
        (_, K_END) => editor.offset = editor.string.len(),
        (_, K_DOWN) => editor.down(),
        (m, _) => {
            let (no_mult, mut times) = match *multiplier {
                Some(n) => (false, n),
                None => (true, 1),
            };
            let mut is_none = false;

            match (*mode, key_event.character) {
                (Normal, '0') if !no_mult => times *= 10,

                (Normal, '1') if no_mult => times = 1,
                (Normal, '1') => times = times * 10 + 1,

                (Normal, '2') if no_mult => times = 2,
                (Normal, '2') => times = times * 10 + 2,

                (Normal, '3') if no_mult => times = 3,
                (Normal, '3') => times = times * 10 + 3,

                (Normal, '4') if no_mult => times = 4,
                (Normal, '4') => times = times * 10 + 4,

                (Normal, '5') if no_mult => times = 5,
                (Normal, '5') => times = times * 10 + 5,

                (Normal, '6') if no_mult => times = 6,
                (Normal, '6') => times = times * 10 + 6,

                (Normal, '7') if no_mult => times = 7,
                (Normal, '7') => times = times * 10 + 7,

                (Normal, '8') if no_mult => times = 8,
                (Normal, '8') => times = times * 10 + 8,

                (Normal, '9') if no_mult => times = 9,
                (Normal, '9') => times = times * 10 + 9,
                (_, _) => {
                    if *is_recording {
                        if key_event.character == ',' {
                            *is_recording = false;
                        } else {
                            period.push(key_event.character);
                        }
                    } else {
                        for _ in 0 .. times {
                            match (m, key_event.character) {
                                (Normal, 'i') => {
                                    *mode = Insert;
                                    *last_change = editor.string.clone();
                                },
                                (Normal, 'h') => editor.left(),
                                (Normal, 'l') => editor.right(),
                                (Normal, 'k') => editor.up(),
                                (Normal, 'j') => editor.down(),
                                (Normal, 'K') => {
                                    for _ in 1..15 {
                                        editor.up();
                                    }
                                },
                                (Normal, 'J') => {
                                    for _ in 1..15 {
                                        editor.down();
                                    }
                                },
                                (Normal, 'g') => editor.offset = 0,
                                (Normal, 'G') => editor.offset = editor.string.len(),
                                (Normal, 'a') => {
                                    editor.right();
                                    *mode = Insert;
                                    *last_change = editor.string.clone();
                                },
                                (Normal, 'x') => editor.delete(window),
                                (Normal, 'X') => editor.backspace(window),
                                (Normal, 'u') => {
                                    editor.offset = 0;
                                    ::core::mem::swap(last_change, &mut editor.string);
                                },
                                (Normal, 'c') => {
                                    ::core::mem::swap(&mut editor.offset, swap);
                                },
                                (Normal, 's') => {
                                    editor.delete(window);
                                    *mode = Insert;
                                },
                                (Normal, 'o') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset < editor.string.len() {
                                        editor.right();
                                    }
                                    editor.insert('\n', window);
                                    *mode = Insert;
                                },
                                (Normal, 'O') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset >= 1 {
                                        editor.left();
                                    }
                                    editor.insert('\n', window);
                                    editor.left();
                                    *mode = Insert;
                                },
                                (Normal, '^') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset <= 0 {
                                        editor.left();
                                    }
                                    editor.right();
                                    while (editor.cur() == ' ' ||
                                          editor.cur() == '\t') &&
                                          editor.offset < editor.string.len() {
                                        editor.right();
                                    }
                                },
                                (Normal, '$') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset < editor.string.len() {
                                        editor.right();
                                    }
                                },
                                (Normal, '0') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset >= 1 {
                                        editor.left();
                                    }
                                    editor.right();
                                },
                                (Normal, 'd') => {
                                    while editor.cur() != '\n' &&
                                          editor.cur() != '\0' &&
                                          editor.offset < editor.string.len() {
                                        editor.delete(window);
                                    }
                                },
                                (Normal, 'w') => {
                                    editor.save(window);
                                },
                                (Normal, 'e') => {
                                    editor.right();
                                    while editor.cur() != '.' &&
                                          editor.cur() != '{' &&
                                          editor.cur() != ',' &&
                                          editor.cur() != ' ' &&
                                          editor.cur() != '}' &&
                                          editor.cur() != '(' &&
                                          editor.cur() != ')' &&
                                          editor.cur() != '[' &&
                                          editor.cur() != ']' &&
                                          editor.cur() != ';' &&
                                          editor.cur() != '"' &&
                                          editor.cur() != '\'' &&
                                          editor.cur() != '\n' &&
                                          editor.offset < editor.string.len() {
                                        editor.right();
                                    }
                                },
                                (Normal, 'b') => {
                                    editor.left();
                                    while editor.cur() != '.' &&
                                          editor.cur() != '{' &&
                                          editor.cur() != ',' &&
                                          editor.cur() != ' ' &&
                                          editor.cur() != '}' &&
                                          editor.cur() != '(' &&
                                          editor.cur() != ')' &&
                                          editor.cur() != '[' &&
                                          editor.cur() != ']' &&
                                          editor.cur() != ';' &&
                                          editor.cur() != '"' &&
                                          editor.cur() != '\'' &&
                                          editor.cur() != '\n' &&
                                          editor.offset >= 1 {
                                        editor.left();
                                    }
                                },
                                (Normal, 'E') => {
                                    editor.right();
                                    while editor.cur() != ' ' && editor.offset < editor.string.len() {
                                        editor.right();
                                    }
                                },
                                (Normal, 'B') => {
                                    editor.left();
                                    while editor.cur() != ' ' && editor.offset >= 1 {
                                        editor.left();
                                    }
                                },
                                (Normal, ',') => {
                                    *is_recording = true;
                                    *period = String::new();
                                },
                                (Normal, '%') => {
                                    match editor.cur() {
                                        '(' | '[' | '{' => {
                                            let mut i = 1;
                                            while i != 0 {
                                                editor.right();
                                                i += match editor.cur() {
                                                    '(' | '[' | '{' => 1,
                                                    ')' | ']' | '}' => -1,
                                                    _ => 0,
                                                };
                                                if editor.offset < editor.string.len() {
                                                    break;
                                                }
                                            }
                                        },
                                        ')' | ']' | '}' => {
                                            let mut i = 1;
                                            while i != 0 {
                                                editor.left();
                                                i += match editor.cur() {
                                                    '(' | '[' | '{' => -1,
                                                    ')' | ']' | '}' => 1,
                                                    _ => 0,
                                                };
                                                if editor.offset >= 0 {
                                                    break;
                                                }
                                            }
                                        },
                                        _ => {},

                                    }
                                },
                                (Normal, '!') => {
                                    for c in period.clone().chars() {
                                        exec(editor, mode, multiplier, last_change, KeyEvent {
                                            character: c,
                                            scancode: 0,
                                            pressed: true,
                                        }, window, swap, period, is_recording);
                                    }
                                },
                                (Insert, '\0') => (),
                                (Insert, c) => {
                                    editor.insert(c, window);
                                },
                                _ => {},
                            }
                        }
                    }
                    is_none = true;
                }
            }

            if !is_none {
                *multiplier = Some(times);
            } else {
                *multiplier = None;
            }
        }
    }
}
