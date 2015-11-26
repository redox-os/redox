/// Public keyboard layouts
/// The layout can be:
/// *   ENGLISH
/// *   FRENCH
pub enum Layout {
    ENGLISH,
    FRENCH,
}

/// Function to get the scancode from the current layout
///
/// # Example
///
/// ```
/// let layout = Layout::ENGLISH;
/// //Get the scancode 'EN'
/// let sc : [[char; 2]; 58] = get_scancode_from_layout(layout);
/// ```
pub fn get_scancode_from_layout(layout: &Layout, scancode: u8) -> [char; 2] {
    match *layout {
        Layout::ENGLISH => SCANCODES_EN[scancode as usize],
        Layout::FRENCH => SCANCODES_FR[scancode as usize],
    }
}

/// Function to return the character associated with the scancode, and the layout
pub fn char_for_scancode(scancode: u8, shift: bool, layout: &Layout) -> char {
    let mut character = '\x00';
    if scancode < 58 {
        let characters: [char; 2] = get_scancode_from_layout(layout, scancode);
        if shift {
            character = characters[1];
        } else {
            // Else...
            character = characters[0];
        }
    }
    character
}

// SCANCODES

/// Scancodes for English keyboards
static SCANCODES_EN: [[char; 2]; 58] = [['\0', '\0'],
                                        ['\x1B', '\x1B'],
                                        ['1', '!'],
                                        ['2', '@'],
                                        ['3', '#'],
                                        ['4', '$'],
                                        ['5', '%'],
                                        ['6', '^'],
                                        ['7', '&'],
                                        ['8', '*'],
                                        ['9', '('],
                                        ['0', ')'],
                                        ['-', '_'],
                                        ['=', '+'],
                                        ['\0', '\0'],
                                        ['\t', '\t'],
                                        ['q', 'Q'],
                                        ['w', 'W'],
                                        ['e', 'E'],
                                        ['r', 'R'],
                                        ['t', 'T'],
                                        ['y', 'Y'],
                                        ['u', 'U'],
                                        ['i', 'I'],
                                        ['o', 'O'],
                                        ['p', 'P'],
                                        ['[', '{'],
                                        [']', '}'],
                                        ['\n', '\n'],
                                        ['\0', '\0'],
                                        ['a', 'A'],
                                        ['s', 'S'],
                                        ['d', 'D'],
                                        ['f', 'F'],
                                        ['g', 'G'],
                                        ['h', 'H'],
                                        ['j', 'J'],
                                        ['k', 'K'],
                                        ['l', 'L'],
                                        [';', ':'],
                                        ['\'', '"'],
                                        ['`', '~'],
                                        ['\0', '\0'],
                                        ['\\', '|'],
                                        ['z', 'Z'],
                                        ['x', 'X'],
                                        ['c', 'C'],
                                        ['v', 'V'],
                                        ['b', 'B'],
                                        ['n', 'N'],
                                        ['m', 'M'],
                                        [',', '<'],
                                        ['.', '>'],
                                        ['/', '?'],
                                        ['\0', '\0'],
                                        ['\0', '\0'],
                                        ['\0', '\0'],
                                        [' ', ' ']];

/// Scancodes for French keyboards
static SCANCODES_FR: [[char; 2]; 58] = [['\0', '\0'],
                                        ['\x1B', '\x1B'],
                                        ['1', '&'],
                                        ['2', 'é'],
                                        ['3', '"'],
                                        ['4', '\''],
                                        ['5', '('],
                                        ['6', '-'],
                                        ['7', 'è'],
                                        ['8', '_'],
                                        ['9', 'ç'],
                                        ['0', 'à'],
                                        ['-', ')'],
                                        ['=', '='],
                                        ['\0', '\0'],
                                        ['\t', '\t'],
                                        ['a', 'A'],
                                        ['z', 'Z'],
                                        ['e', 'E'],
                                        ['r', 'R'],
                                        ['t', 'T'],
                                        ['y', 'Y'],
                                        ['u', 'U'],
                                        ['i', 'I'],
                                        ['o', 'O'],
                                        ['p', 'P'],
                                        ['^', '¨'],
                                        ['$', '£'],
                                        ['\n', '\n'],
                                        ['\0', '\0'],
                                        ['q', 'Q'],
                                        ['s', 'S'],
                                        ['d', 'D'],
                                        ['f', 'F'],
                                        ['g', 'G'],
                                        ['h', 'H'],
                                        ['j', 'J'],
                                        ['k', 'K'],
                                        ['l', 'L'],
                                        ['m', 'M'],
                                        ['ù', '%'],
                                        ['*', 'µ'],
                                        ['\0', '\0'],
                                        ['<', '>'],
                                        ['w', 'W'],
                                        ['x', 'X'],
                                        ['c', 'C'],
                                        ['v', 'V'],
                                        ['b', 'B'],
                                        ['n', 'N'],
                                        [',', '?'],
                                        [';', '.'],
                                        [':', '/'],
                                        ['!', '§'],
                                        ['\0', '\0'],
                                        ['\0', '\0'],
                                        ['\0', '\0'],
                                        [' ', ' ']];
