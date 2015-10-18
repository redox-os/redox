use super::*;
use redox::*;

/// Get the next instruction
// TODO: Should this be an iterator instead?
pub fn next_inst(editor: &mut Editor) -> Inst {
    let mut n = 0;
    let mut shifted = false;

    // TODO: Make the switch to normal mode shift more well-coded.
    loop {
        if let EventOption::Key(k) = editor.window.poll().unwrap_or(Event::new()).to_option() {
            let c = k.character;
            if shifted && c == '\u{000E}' {
                editor.cursor_mut().mode = Mode::Command(CommandMode::Normal);
            } else {
                if k.pressed {
                    if c == '\u{000E}' {
                        shifted = true;
                    } else {
                        shifted = false;
                        match editor.cursor().mode {
                            Mode::Primitive(_) => {
                                return Inst(0, c);
                            },
                            Mode::Command(_) => {
                                n = match c {
                                    '0' if n != 0 => n * 10,
                                    '1'           => n * 10 + 1,
                                    '2'           => n * 10 + 2,
                                    '3'           => n * 10 + 3,
                                    '4'           => n * 10 + 4,
                                    '5'           => n * 10 + 5,
                                    '6'           => n * 10 + 6,
                                    '7'           => n * 10 + 7,
                                    '8'           => n * 10 + 8,
                                    '9'           => n * 10 + 9,
                                    _             => {

                                        return Inst(if n == 0 { 1 } else { n }, c);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    unreachable!()
}
