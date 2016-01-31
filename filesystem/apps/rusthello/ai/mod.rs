use interface;
use reversi;

mod ai_brute;
mod ai_heavy;
mod ai_stable;

const DEPTH: u8 = 7;
const ENDING_DEPTH: u8 = 13;



/// It represents the different kind of player who can take part to the game.
#[derive(Clone)]
pub enum Opponent {
    Human,
    AiEasy,
    AiMedium,
    AiHard,
}



impl Opponent {

    /// It produces the new move from each kind of Opponent.
    pub fn make_move(&self, game: &reversi::Game) -> interface::Command {

        if let reversi::Status::Ended = game.get_status() {
            panic!("make_move called on ended game!");
        }

        if let Opponent::Human = *self {
			interface::human_make_move(game)
		} else {
			let (row, col) = ai_make_move(game, (*self).clone());

			interface::print_move(game, (row, col));

			interface::Command::Move(row, col)
        }
    }
}



fn ai_make_move(game: &reversi::Game, opponent: Opponent) -> (usize, usize) {

    let mut num_moves = 0;
    let mut forced_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
    let mut game_after_move = game.clone();

    // To save computation time, first check whether the move is forced.
    for row in 0..reversi::BOARD_SIZE {
        for col in 0..reversi::BOARD_SIZE {
            if game_after_move.make_move((row, col)) {
                num_moves += 1;
                forced_move = (row, col);
                game_after_move = game.clone();
            }
        }
    }

    match num_moves {
        0 => panic!("No valid move is possible!"),
        1 => forced_move,
        _ => {

			let find_best_move: fn(&reversi::Game, u8) -> (usize, usize) = match opponent {
    			Opponent::AiEasy   => ai_brute::find_best_move,
    			Opponent::AiMedium => ai_heavy::find_best_move,
    			Opponent::AiHard   => ai_stable::find_best_move,
				Opponent::Human    => panic!("A human is not an AI!")
			};

            if game.get_tempo() + ENDING_DEPTH >= 64 {
                find_best_move(game, (reversi::BOARD_SIZE * reversi::BOARD_SIZE) as u8 - game.get_tempo())
            } else {
                find_best_move(game, DEPTH)
            }
        }
    }
}
