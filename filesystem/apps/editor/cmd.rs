use redox::*;

use super::Mode;
use super::Mode::*;
use super::Editor;

pub fn exec(editor: &mut Editor, mode: &mut Mode, multiplier: &mut Option<u32>, last_change: &mut String, key_event: KeyEvent, window: &mut Window) {
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

            match key_event.character {
                '0' if !no_mult => times *= 10,

                '1' if no_mult => times = 1,
                '1' => times = times * 10 + 1,

                '2' if no_mult => times = 2,
                '2' => times = times * 10 + 2,

                '3' if no_mult => times = 3,
                '3' => times = times * 10 + 3,

                '4' if no_mult => times = 4,
                '4' => times = times * 10 + 4,

                '5' if no_mult => times = 5,
                '5' => times = times * 10 + 5,

                '6' if no_mult => times = 6,
                '6' => times = times * 10 + 6,

                '7' if no_mult => times = 7,
                '7' => times = times * 10 + 7,

                '8' if no_mult => times = 8,
                '8' => times = times * 10 + 8,

                '9' if no_mult => times = 9,
                '9' => times = times * 10 + 9,
                _ => {
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
                            (Normal, '$') => {
                                let mut new_offset = editor.string.len();
                                for i in editor.offset..editor.string.len() {
                                    match editor.string.as_bytes()[i] {
                                        0 => break,
                                        10 => {
                                            new_offset = i;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                editor.offset = new_offset;
                            },
                            (Normal, '0') => {

                                let mut new_offset = 0;
                                for i in 2..editor.offset {
                                    match editor.string.as_bytes()[editor.offset - i] {
                                        0 => break,
                                        10 => {
                                            new_offset = editor.offset - i + 1;
                                            break;
                                        }
                                        _ => (),
                                    }
                                }
                                editor.offset = new_offset;
                            },
                            (Insert, '\0') => (),
                            (Insert, _) => {
                                window.set_title(&format!("{}{}{}","Editor (", &editor.url, ") Changed"));
                                editor.string = editor.string[0 .. editor.offset].to_string() +
                                    &key_event.character.to_string() +
                                    &editor.string[editor.offset .. editor.string.len()];
                                editor.offset += 1;
                            },
                            _ => {},
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
