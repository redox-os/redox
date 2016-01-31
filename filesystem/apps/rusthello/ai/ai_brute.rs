use reversi;

use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const BONUS_TURN: i16 = 3;



pub fn find_best_move(game: &reversi::Game, depth: u8) -> (usize, usize) {

    if let reversi::Status::Running { current_player } = game.get_status() {

        let mut best_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
        let mut moves_num: u8 = 0;
        let mut best_score: i16 = match current_player {
            reversi::Player::Light => LIGHT_STARTING_SCORE,
            reversi::Player::Dark  => DARK_STARTING_SCORE,
        };

        let (tx, rx): (Sender<((usize, usize), i16)>, Receiver<((usize, usize), i16)>) = mpsc::channel();

        let mut game_after_move = game.clone();

        for row in 0..reversi::BOARD_SIZE {
            for col in 0..reversi::BOARD_SIZE {
                if game_after_move.make_move((row, col)) {

                    moves_num +=1;
                    let thread_tx = tx.clone();

                    thread::spawn(move || {
                        let current_score = eval(&game_after_move, depth - 1);
                        thread_tx.send(((row, col), current_score)).unwrap();
                    });

                    game_after_move = game.clone();
                }
            }
        }

        for _ in 0..moves_num {
            let (current_move, current_score) = rx.recv().ok().expect("Could not receive answer");

            match current_player {
                reversi::Player::Light => {
                    if current_score > best_score {
                        best_move = current_move;
                        best_score = current_score;
                    }
                }
                reversi::Player::Dark  => {
                    if current_score < best_score {
                        best_move = current_move;
                        best_score = current_score;
                    }
                }
            }
        }

        best_move
    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}



fn eval(game: &reversi::Game, depth: u8) -> i16 {

    match game.get_status() {
        reversi::Status::Running { current_player } => {
            if depth == 0 {
                match current_player {
                    reversi::Player::Light => game.get_score_diff() + BONUS_TURN,
                    reversi::Player::Dark  => game.get_score_diff() - BONUS_TURN,
                }
            } else {
                match current_player {
                    reversi::Player::Light => {
                        let mut best_score: i16 = LIGHT_STARTING_SCORE;
                        let mut current_score: i16;
                        let mut game_after_move = game.clone();
                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {
                                    current_score = eval(&game_after_move, depth - 1);
                                    if current_score > best_score {
                                        best_score = current_score;
                                    }
                                    game_after_move = game.clone();
                                }
                            }
                        }
                        best_score
                    }
                    reversi::Player::Dark => {
                        let mut best_score: i16 =  DARK_STARTING_SCORE;
                        let mut current_score: i16;
                        let mut game_after_move = game.clone();
                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {
                                    current_score = eval(&game_after_move, depth - 1);
                                    if current_score < best_score {
                                        best_score = current_score;
                                    }
                                    game_after_move = game.clone();
                                }
                            }
                        }
                        best_score
                    }
                }
            }
        }
        reversi::Status::Ended => {
            game.get_score_diff()*64
        }
    }
}
