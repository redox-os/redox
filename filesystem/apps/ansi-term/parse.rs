use escape::OutputEscapeCode;

pub enum Action {
    Char(char),
    Control(OutputEscapeCode),
}

pub struct ActionIter<I> {
    iter: I,
}

impl<I: Iterator<Item = char>> ActionIter {
    fn get_escape_code(&mut self) -> OutputEscapeCode {
        let params = vec![0];
        let mut question_marked = false;

        for c in self.iter {
            match c {
                '?' => question_marked = true,
                '0'...'9' => {
                    let cur = params.last_mut().unwrap();
                    let n = (c as u8) - b'0';

                    *cur = *cur * 10 + n as u16;
                }
                ';' => params.push(0),
                _ => {
                    match c {
                        // Navigation
                        'A' if !params.is_empty() => return OutputEscapeCode::CursorUp(params[0]),
                        'A' => return OutputEscapeCode::CursorUp(1),
                        'B' if !params.is_empty() => return OutputEscapeCode::CursorDown(params[0]),
                        'B' => return OutputEscapeCode::CursorDown(1),
                        'C' if !params.is_empty() => {
                            return OutputEscapeCode::CursorRight(params[0])
                        }
                        'C' => return OutputEscapeCode::CursorRight(1),
                        'D' if !params.is_empty() => return OutputEscapeCode::CursorLeft(params[0]),
                        'D' => return OutputEscapeCode::CursorLeft(1),
                        'E' if !params.is_empty() => return OutputEscapeCode::NextLine(params[0]),
                        'E' => return OutputEscapeCode::NextLine(1),
                        'F' if !params.is_empty() => {
                            return OutputEscapeCode::PreviousLine(params[0])
                        }
                        'F' => return OutputEscapeCode::PreviousLine(1),
                        // Goto
                        'G' if !params.is_empty() => return params[0],
                        'G' => return 1,
                        'H' | 'f' if params.len() >= 2 => {
                            return OutputEscapeCode::Goto(params[0], params[1])
                        }
                        'H' | 'f' if params.len() == 1 => {
                            return OutputEscapeCode::Goto(params[0], 1)
                        }
                        'H' | 'f' => return OutputEscapeCode::GotoStart,
                        // Erase
                        'J' if params.is_empty() || params[0] == 0 => {
                            return OutputEscapeCode::EraseAfter
                        }
                        'J' if params[0] == 1 => return OutputEscapeCode::EraseBefore,
                        'J' if params[0] == 2 => return OutputEscapeCode::EraseAll,
                        // Erase in line
                        'K' if params.is_empty() || params[0] == 0 => {
                            return OutputEscapeCode::EraseLineAfter
                        }
                        'K' if params[0] == 1 => return OutputEscapeCode::EraseLineBefore,
                        'K' if params[0] == 2 => return OutputEscapeCode::EraseLine,
                        // Scrolling
                        'S' if params.is_empty() => return OutputEscapeCode::ScrollUp(1),
                        'S' => return OutputEscapeCode::ScrollUp(params[0]),
                        'T' if params.is_empty() => return OutputEscapeCode::ScrollDown(1),
                        'T' => return OutputEscapeCode::ScrollDown(params[0]),
                        // Graphics
                        'm' if params.is_empty() => return OutputEscapeCode::Rendition(0),
                        'm' => return OutputEscapeCode::Rendition(params[0]),
                        's' => return OutputEscapeCode::SaveCursor,
                        'u' => return OutputEscapeCode::RestoreCursor,
                        'l' if question_marked && vec[0] == 25 => OutputEscapeCode::HideCursor,
                        'h' if question_marked && vec[0] == 25 => OutputEscapeCode::ShowCursor,
                        _ => unimplemented!(),
                    }
                }
            }
        }

        OutputEscapeCode::None
    }
}

impl<I: Iterator<Item = char>> Iterator for ActionIter {
    type Item = Action;

    fn next(&mut self) -> Option<Action> {
        for c in self.iter {
            match c {
                '\x1B' => return Action::Control(self.get_escape_code()),
                '\n' => return Action::Control(OutputEscapeCode::NewLine),
                '\r' => return Action::Control(OutputEscapeCode::CarriageReturn),
                _ => Action::Char(c),
            }
        }
    }
}
