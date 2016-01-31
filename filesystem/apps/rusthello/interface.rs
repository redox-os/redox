// This module provides interface functionalities and manages all the input/output part of the program

use std::cmp::Ordering;
use std::io::{self, Write};
use ai;
use reversi;

pub enum Command {
	Move(usize, usize),
	Undo,
	Quit
}


pub fn start_game() -> Option<(ai::Opponent, ai::Opponent)> {

    // Print a start message
    println!("\n\n\n\t         RUSThello");
    println!("\t         ● ○ ● ○ ●");
    println!("\t   a simple Othello game");
    println!("\t written in Rust with love");
    println!("\t       Redox Edition");
    println!("\t          v 1.0.0\n\n\n");

    println!(" 1 Human");
    println!(" 2 AI - Weak");
    println!(" 3 AI - Medium");
    println!(" 4 AI - Strong");
    println!(" 0 Quit\n");

    let light = select_opponent("Light");
	if light.is_none() {
		return Option::None;
	}

    let dark = select_opponent("Dark");
	if dark.is_none() {
		return Option::None;
	}

    println!("\n\n\nStarting new game…");
    println!("Type 'undo' instead of a move to undo last move.");
    println!("Type 'quit' instead of a move to abandon the game.");

    Option::Some((light.unwrap(), dark.unwrap()))
}


fn select_opponent(name: &str) -> Option<ai::Opponent> {

    loop {
        print!("Select {} player: ", name);

        // Read the input
        let _ = io::stdout().flush();

        let mut input = String::new();

        io::stdin().read_line(&mut input)
            .ok()
            .expect("failed to read line");

		let input = input.trim().parse::<u32>();

		if let Result::Ok(player_num) = input {
			return match player_num {
				0 => Option::None,
				1 => Option::Some(ai::Opponent::Human),
				2 => Option::Some(ai::Opponent::AiEasy),
				3 => Option::Some(ai::Opponent::AiMedium),
				4 => Option::Some(ai::Opponent::AiHard),
            	_ => {
            	    println!("This is not a valid choice, please input a valid player.");
            	    continue;
            	}
        	}
		} else {
			println!("This is not a valid choice, please input a valid player.");
			continue;
		}
    }
}



/// draw_board draws the board (using text characters) in a pleasant-looking way, converting the board in a string (board_to_string) and then printing this.
pub fn draw_board(game: &reversi::Game) {

    let board = game.get_board();

    // Declare board_to_string and add column reference at the top
    let mut board_to_string: String = "\n\n\n\t   a  b  c  d  e  f  g  h\n".to_string();

    // For every row add a row reference to the left
    for (row, row_array) in board.iter().enumerate() {
        board_to_string.push('\t');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push(' ');

        // For every column, add the appropriate character depending on the content of the current cell
        for (col, cell) in row_array.iter().enumerate() {

            match *cell {
                // Light and Dark cells are represented by white and black bullets
                reversi::Cell::Taken { disk: reversi::Player::Light } => board_to_string.push_str(" ○ "),
                reversi::Cell::Taken { disk: reversi::Player::Dark }  => board_to_string.push_str(" ● "),

                // An empty cell will display a plus or a multiplication sign if the current player can move in that cell
                // or a little central dot otherwise
                reversi::Cell::Empty => {
                    if game.check_move((row, col)) {
                        if let reversi::Status::Running { current_player } = game.get_status() {
                            match current_player {
                                reversi::Player::Light => board_to_string.push_str(" + "),
                                reversi::Player::Dark  => board_to_string.push_str(" × "),
                            }
                        }
                    } else {
                        board_to_string.push_str(" ∙ ");
                    }
                }
            }
        }

        // Add a row reference to the right
        board_to_string.push(' ');
        board_to_string.push_str(&(row + 1).to_string());
        board_to_string.push('\n');
    }

    // Add column reference at the bottom
    board_to_string.push_str("\t   a  b  c  d  e  f  g  h\n");

    // Print board
    println!("{}", board_to_string);

    // Print current score and game info
    let (score_light, score_dark) = game.get_score();

    match game.get_status() {
        reversi::Status::Running { current_player } => {
            match current_player {
                reversi::Player::Light => println!("\t        {:>2} ○ << ● {:<2}\n", score_light, score_dark),
                reversi::Player::Dark  => println!("\t        {:>2} ○ >> ● {:<2}\n", score_light, score_dark),
            }
        }
        reversi::Status::Ended => {
            println!("\t        {:>2} ○    ● {:<2}\n", score_light, score_dark);
            match score_light.cmp(&score_dark) {
                Ordering::Greater => println!("Light wins!"),
                Ordering::Less    => println!("Dark wins!"),
                Ordering::Equal   => println!("Draw!"),
            }
        }
    }
}

pub fn print_move(game: &reversi::Game, (row, col): (usize, usize)) {

    let char_col = (('a' as u8) + (col as u8)) as char;
    if let reversi::Status::Running { current_player } = game.get_status() {
        match current_player {
            reversi::Player::Light => println!("○ Light moves: {}{}", char_col, row + 1),
            reversi::Player::Dark  => println!("● Dark moves: {}{}",  char_col, row + 1),
        }
    }
}


/// It get_status a human player's input and convert it into a move. If the move if illegal, it ask for another input until the given move is a legal one.
pub fn human_make_move(game: &reversi::Game) -> Command {

    if let reversi::Status::Running { current_player } = game.get_status() {
        match current_player {
            reversi::Player::Light => print!("○ Light moves: "),
            reversi::Player::Dark  => print!("● Dark moves: "),
        }
    }
    let mut next_move = String::new();
    let (mut row, mut col): (usize, usize) = (9, 9);

    loop {

        // Read the input
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut next_move)
            .ok()
            .expect("Failed to read line");

		// Quitting the game
		match &*(next_move.trim().to_lowercase()) {
			"quit" => { return Command::Quit; }
			"undo" => { return Command::Undo; }
			_ => {
				// Every char in the input string which is a digit (0--9, a(A)--z(Z)) is interpreted as a row or column
				// and assigned to the relevant variable
				for curr_char in next_move.chars() {
				    if let Some(digit) = curr_char.to_digit(36) {
				        if digit <= 9 {
				            // rows have to be corrected with a -1, as the first row (labelled '1') has index 0 and so on...
				            row = ( digit - 1 ) as usize;
				        } else if digit >= 10 {
				            // rows have to be corrected with a -10, as the first column (labelled 'a') has index 0 and so on...
				            col = ( digit - 10 ) as usize;
				        }
				    }
				}

				// If the given move is valid, return it; otherwise complain about the illegal move and ask for another one
				if game.check_move((row, col)) {
				    return Command::Move(row, col);
				} else {
				    print!("Illegal move, try again: ");
				}
			}
		}
    }
}



// Print a last message before a player quits the game
pub fn quitting_message(coward: reversi::Player) {
    match coward {
        reversi::Player::Light => println!("Light is running away, the coward!"),
        reversi::Player::Dark  => println!("Dark is running away, the coward!"),
    }
}

// Print a last message when 'undo' is not possible
pub fn no_undo_message(undecided: reversi::Player) {
	match undecided {
        reversi::Player::Light => println!("There is no move Light can undo."),
        reversi::Player::Dark  => println!("There is no move Dark can undo."),
    }
}
