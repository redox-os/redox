//! A simple Othello game written in Rust with love
//! by Enrico Ghiorzi



// Import modules
mod reversi;
mod interface;
mod ai;



pub fn main() {

    // Create a new game and two opponents

    let players = interface::start_game();

	if players.is_none() {
		return;
	}

	let (light, dark) = players.unwrap();

    let mut game = reversi::Game::new();

	let mut hystory: Vec<reversi::Game> = Vec::new();

	let mut draw_board = true;

    // Proceed with turn after turn till the endgame
    'new_turn: loop {
        // Draw the current board and game info
        if draw_board {
			interface::draw_board(&game);
		}

        // Depending on the status of the game, proceed with the next turn or declare the winner
        match game.get_status() {

            // If the game is running, get the coordinates of the new move from the next player
            reversi::Status::Running { current_player } => {

                let action = match current_player {
                    reversi::Player::Light => light.make_move(&game),
                    reversi::Player::Dark  => dark.make_move(&game),
                };

				match action {
                	// Quitting RUSThello
					interface::Command::Quit => {
	                    interface::quitting_message(current_player);
	                    break;
					}

					// Manages hystory
					interface::Command::Undo => {
                        let mut recovery: Vec<reversi::Game> = Vec::new();

                        while let Some(previous_game) = hystory.pop() {
                            recovery.push(previous_game.clone());
                            if let reversi::Status::Running { current_player: previous_player } = previous_game.get_status() {
                                if previous_player == current_player {
                                    game = previous_game;
                                    draw_board = true;
                                    continue 'new_turn;
                                }
                            }
                        }

                        while let Some(recovered_game) = recovery.pop() {
                            hystory.push(recovered_game.clone());
                        }

                        draw_board = false;
						interface::no_undo_message(current_player);
					}

		            // If the new move is valid, perform it; otherwise panic
		            // NB: opponents's make_move method is responsible for returning a legal move
		            //     so the program should never print this message unless something goes horribly wrong
					interface::Command::Move(row, col) => {

						if game.check_move((row, col)) {
                            hystory.push(game.clone());
                            game.make_move((row, col));
							draw_board = true;
						} else {
							panic!("Invalid move sent to main!");
						}
					}
				}
            }

            // If the game is ended, exit the loop
            reversi::Status::Ended => {
                break;
            }
        }
    }
}
