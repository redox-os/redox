/// Public keyboard layouts
/// The layout can be:
/// *   English
/// *   Colemak
/// *   French
/// *   German
pub enum Layout {
    English,
    Colemak,
    French,
    German,
}

/// Function to get the scancode from the current layout
///
/// # Example
///
/// ```
/// let layout = Layout::English;
/// //Get the scancode 'EN'
/// let sc : [[char; 3]; 58] = get_scancode_from_layout(layout);
/// ```
pub fn get_scancode_from_layout(layout: &Layout, scancode: u8) -> [char; 3] {
    match *layout {
        Layout::English => SCANCODES_EN[scancode as usize],
        Layout::Colemak => SCANCODES_CO[scancode as usize],
        Layout::French => SCANCODES_FR[scancode as usize],
        Layout::German => SCANCODES_DE[scancode as usize],
    }
}

fn get_special_keys_from_layout(layout: &Layout, scancode: u8) -> [char; 3] {
    let keys: &[(u8, [char; 3])] = match *layout {
        Layout::English => SCANCODES_EXTRA_EN,
        Layout::Colemak => SCANCODES_EXTRA_CO,
        Layout::French => SCANCODES_EXTRA_FR,
        Layout::German => SCANCODES_EXTRA_DE,
    };
    match keys.iter().filter(|&&(code, _)| code == scancode).next() {
        Some(&(_, keys)) => keys,
        None => ['\0', '\0', '\0'],
    }
}


/// Function to return the character associated with the scancode, and the layout
pub fn char_for_scancode(scancode: u8, shift: bool, altgr: bool, layout: &Layout) -> char {
    let character;

    let characters = if scancode < 58 {
        get_scancode_from_layout(layout, scancode)
    } else {
        get_special_keys_from_layout(layout, scancode)
    };

    if altgr {
        character = characters[2];
    } else if shift {
        character = characters[1];
    } else {
        character = characters[0];
    }
    
    character
}

// SCANCODES

/// Scancodes for English keyboards
static SCANCODES_EN: [[char; 3]; 58] = [['\0', '\0', '\0'],
                                        ['\x1B', '\x1B', '\x1B'],
                                        ['1', '!', '1'],
                                        ['2', '@', '2'],
                                        ['3', '#', '3'],
                                        ['4', '$', '4'],
                                        ['5', '%', '5'],
                                        ['6', '^', '6'],
                                        ['7', '&', '7'],
                                        ['8', '*', '8'],
                                        ['9', '(', '9'],
                                        ['0', ')', '0'],
                                        ['-', '_', '-'],
                                        ['=', '+', '='],
                                        ['\0', '\0', '\0'],
                                        ['\t', '\t', '\t'],
                                        ['q', 'Q', 'q'],
                                        ['w', 'W', 'w'],
                                        ['e', 'E', 'e'],
                                        ['r', 'R', 'r'],
                                        ['t', 'T', 't'],
                                        ['y', 'Y', 'y'],
                                        ['u', 'U', 'u'],
                                        ['i', 'I', 'i'],
                                        ['o', 'O', 'o'],
                                        ['p', 'P', 'p'],
                                        ['[', '{', '['],
                                        [']', '}', ']'],
                                        ['\n', '\n', '\n'],
                                        ['\0', '\0', '\0'],
                                        ['a', 'A', 'a'],
                                        ['s', 'S', 's'],
                                        ['d', 'D', 'd'],
                                        ['f', 'F', 'f'],
                                        ['g', 'G', 'g'],
                                        ['h', 'H', 'h'],
                                        ['j', 'J', 'j'],
                                        ['k', 'K', 'k'],
                                        ['l', 'L', 'l'],
                                        [';', ':', ';'],
                                        ['\'', '"', '\''],
                                        ['`', '~', '`'],
                                        ['\0', '\0', '\0'],
                                        ['\\', '|', '\\'],
                                        ['z', 'Z', 'z'],
                                        ['x', 'X', 'x'],
                                        ['c', 'C', 'c'],
                                        ['v', 'V', 'v'],
                                        ['b', 'B', 'b'],
                                        ['n', 'N', 'n'],
                                        ['m', 'M', 'm'],
                                        [',', '<', ','],
                                        ['.', '>', '.'],
                                        ['/', '?', '/'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        [' ', ' ', ' ']];

/// Special keys, not present on every keyboard
static SCANCODES_EXTRA_EN: &'static [(u8, [char; 3])] = &[];

/// Scancodes for Colemak keyboards
static SCANCODES_CO: [[char; 3]; 58] = [['\0', '\0', '\0'],
                                        ['\x1B', '\x1B', '\x1B'],
                                        ['1', '!', '1'],
                                        ['2', '@', '2'],
                                        ['3', '#', '3'],
                                        ['4', '$', '4'],
                                        ['5', '%', '5'],
                                        ['6', '^', '6'],
                                        ['7', '&', '7'],
                                        ['8', '*', '8'],
                                        ['9', '(', '9'],
                                        ['0', ')', '0'],
                                        ['-', '_', '-'],
                                        ['=', '+', '='],
                                        ['\0', '\0', '\0'],
                                        ['\t', '\t', '\t'],
                                        ['q', 'Q', 'q'],
                                        ['w', 'W', 'w'],
                                        ['f', 'F', 'f'],
                                        ['p', 'P', 'p'],
                                        ['g', 'G', 'g'],
                                        ['j', 'J', 'j'],
                                        ['l', 'L', 'l'],
                                        ['u', 'U', 'u'],
                                        ['y', 'Y', 'y'],
                                        [';', ':', ';'],
                                        ['[', '{', '['],
                                        [']', '}', ']'],
                                        ['\n', '\n', '\n'],
                                        ['\0', '\0', '\0'],
                                        ['a', 'A', 'a'],
                                        ['r', 'R', 'r'],
                                        ['s', 'S', 's'],
                                        ['t', 'T', 't'],
                                        ['d', 'D', 'd'],
                                        ['h', 'H', 'h'],
                                        ['n', 'N', 'n'],
                                        ['e', 'E', 'e'],
                                        ['i', 'I', 'i'],
                                        ['o', 'O', 'o'],
                                        ['\'', '"', '\''],
                                        ['`', '~', '`'],
                                        ['\0', '\0', '\0'],
                                        ['\\', '|', '\\'],
                                        ['z', 'Z', 'z'],
                                        ['x', 'X', 'x'],
                                        ['c', 'C', 'c'],
                                        ['v', 'V', 'v'],
                                        ['b', 'B', 'b'],
                                        ['k', 'K', 'k'],
                                        ['m', 'M', 'm'],
                                        [',', '<', ','],
                                        ['.', '>', '.'],
                                        ['/', '?', '/'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        [' ', ' ', ' ']];

/// Special keys, not present on every keyboard
static SCANCODES_EXTRA_CO: &'static [(u8, [char; 3])] = &[];

/// Scancodes for French keyboards
static SCANCODES_FR: [[char; 3]; 58] = [['\0', '\0', '\0'],
                                        ['\x1B', '\x1B', '\0'],
                                        ['1', '&', '1'],
                                        ['2', 'é', '2'],
                                        ['3', '"', '3'],
                                        ['4', '\'', '4'],
                                        ['5', '(', '5'],
                                        ['6', '-', '6'],
                                        ['7', 'è', '7'],
                                        ['8', '_', '8'],
                                        ['9', 'ç', '9'],
                                        ['0', 'à', '0'],
                                        ['-', ')', '-'],
                                        ['=', '=', '='],
                                        ['\0', '\0', '\0'],
                                        ['\t', '\t', '\t'],
                                        ['a', 'A', 'a'],
                                        ['z', 'Z', 'z'],
                                        ['e', 'E', 'e'],
                                        ['r', 'R', 'r'],
                                        ['t', 'T', 't'],
                                        ['y', 'Y', 'y'],
                                        ['u', 'U', 'u'],
                                        ['i', 'I', 'i'],
                                        ['o', 'O', 'o'],
                                        ['p', 'P', 'p'],
                                        ['^', '¨', '^'],
                                        ['$', '£', '$'],
                                        ['\n', '\n', '\n'],
                                        ['\0', '\0', '\0'],
                                        ['q', 'Q', 'q'],
                                        ['s', 'S', 's'],
                                        ['d', 'D', 'd'],
                                        ['f', 'F', 'f'],
                                        ['g', 'G', 'g'],
                                        ['h', 'H', 'h'],
                                        ['j', 'J', 'j'],
                                        ['k', 'K', 'k'],
                                        ['l', 'L', 'l'],
                                        ['m', 'M', 'm'],
                                        ['ù', '%', 'ù'],
                                        ['*', 'µ', '*'],
                                        ['\0', '\0', '\0'],
                                        ['<', '>', '|'],
                                        ['w', 'W', 'w'],
                                        ['x', 'X', 'x'],
                                        ['c', 'C', 'c'],
                                        ['v', 'V', 'v'],
                                        ['b', 'B', 'b'],
                                        ['n', 'N', 'n'],
                                        [',', '?', ','],
                                        [';', '.', ';'],
                                        [':', '/', ':'],
                                        ['!', '§', '!'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        [' ', ' ', ' ']];

/// Special keys, not present on every keyboard
static SCANCODES_EXTRA_FR: &'static [(u8, [char; 3])] = &[];

/// Scancodes for German keyboards
static SCANCODES_DE: [[char; 3]; 58] = [['\0', '\0', '\0'],
                                        ['\x1B', '\x1B', '\x1B'],
                                        ['1', '!', '1'],
                                        ['2', '"', '²'],
                                        ['3', '§', '³'],
                                        ['4', '$', '4'],
                                        ['5', '%', '%'],
                                        ['6', '&', '6'],
                                        ['7', '/', '{'],
                                        ['8', '(', '['],
                                        ['9', ')', ']'],
                                        ['0', '=', '}'],
                                        ['ß', '?', '\\'],
                                        ['\'', '`', '\''],
                                        ['\0', '\0', '\0'],
                                        ['\t', '\t', '\t'],
                                        ['q', 'Q', '@'],
                                        ['w', 'W', 'w'],
                                        ['e', 'E', '€'],
                                        ['r', 'R', 'r'],
                                        ['t', 'T', 't'],
                                        ['z', 'Z', 'z'],
                                        ['u', 'U', 'u'],
                                        ['i', 'I', 'i'],
                                        ['o', 'O', 'o'],
                                        ['p', 'P', 'p'],
                                        ['ü', 'Ü', 'ü'],
                                        ['+', '*', '~'],
                                        ['\n', '\n', '\n'],
                                        ['\0', '\0', '\0'],
                                        ['a', 'A', 'a'],
                                        ['s', 'S', 's'],
                                        ['d', 'D', 'd'],
                                        ['f', 'F', 'f'],
                                        ['g', 'G', 'g'],
                                        ['h', 'H', 'h'],
                                        ['j', 'J', 'j'],
                                        ['k', 'K', 'k'],
                                        ['l', 'L', 'l'],
                                        ['ö', 'Ö', 'ö'],
                                        ['ä', 'Ä', 'ä'],
                                        ['^', '°', '^'],
                                        ['\0', '\0', '\0'],
                                        ['#', '\'', '#'],
                                        ['y', 'Y', 'y'],
                                        ['x', 'X', 'x'],
                                        ['c', 'C', 'c'],
                                        ['v', 'V', 'v'],
                                        ['b', 'B', 'b'],
                                        ['n', 'N', 'n'],
                                        ['m', 'M', 'µ'],
                                        [',', ';', ','],
                                        ['.', ':', '.'],
                                        ['-', '_', '-'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        ['\0', '\0', '\0'],
                                        [' ', ' ', ' ']];

/// Special keys, not present on every keyboard
static SCANCODES_EXTRA_DE: &'static [(u8, [char; 3])] = &[(0x56, ['<', '>', '|'])];
