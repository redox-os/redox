use reversi;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};

const LIGHT_STARTING_SCORE: i16 = -10_000;
const DARK_STARTING_SCORE:  i16 =  10_000;

const RANDOMNESS: i16 = 3;

const BONUS_TURN: i16 = 3;

const MOBILITY: i16 = 2;

#[derive(Clone)]
enum EvalReturn {
    Ended,
    Running { fixed_cells_board: reversi::Board }
}



pub fn find_best_move(game: &reversi::Game, depth: u8) -> (usize, usize) {

    if let reversi::Status::Running { current_player } = game.get_status() {

        if depth > 0 {

            let mut best_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
            let mut best_score: i16 = match current_player {
                reversi::Player::Light => LIGHT_STARTING_SCORE,
                reversi::Player::Dark  => DARK_STARTING_SCORE,
            };

            let mut best_end_move: (usize, usize) = (reversi::BOARD_SIZE, reversi::BOARD_SIZE);
            let mut best_end_score: i16 = best_score;

            let mut num_moves: u8 = 0;
            let mut end_game: bool = true;

            let (tx, rx): (Sender<((usize, usize), (i16, EvalReturn))>, Receiver<((usize, usize), (i16, EvalReturn))>) = mpsc::channel();

            let mut game_after_move = game.clone();

            for row in 0..reversi::BOARD_SIZE {
                for col in 0..reversi::BOARD_SIZE {
                    if game_after_move.make_move((row, col)) {

                        num_moves +=1;
                        let thread_tx = tx.clone();

                        thread::spawn(move || {
                            thread_tx.send(( (row, col), eval(&game_after_move, depth - 1) )).unwrap();
                        });

                        game_after_move = game.clone();

                    }
                }
            }

            for _ in 0..num_moves {
                let (current_move, (current_score, current_eval_return)) = rx.recv().ok().expect("Could not receive answer");

                match current_player {
                    reversi::Player::Light => {
                        if let EvalReturn::Ended = current_eval_return {
                            if current_score > best_end_score {
                                best_end_score = current_score;
                                best_end_move = current_move;
                            }
                        } else {
                            if current_score + RANDOMNESS > best_score {
                                best_score = current_score;
                                best_move = current_move;
                                end_game = false;
                            }
                        }
                    }
                    reversi::Player::Dark  => {
                        if let EvalReturn::Ended = current_eval_return {
                            if current_score < best_end_score {
                                best_end_score = current_score;
                                best_end_move = current_move;
                            }
                        } else {
                            if current_score - RANDOMNESS < best_score {
                                best_score = current_score;
                                best_move = current_move;
                                end_game = false;
                            }
                        }
                    }
                }
            }

            match current_player {
                reversi::Player::Light  => {
                    if best_end_score > 0 || (best_end_score == 0 && best_score < 0) || end_game {
                        return best_end_move;
                    } else {
                        return best_move;
                    }
                }
                reversi::Player::Dark  => {
                    if best_end_score < 0 || (best_end_score == 0 && best_score > 0) || end_game {
                        return best_end_move;
                    } else {
                        return best_move;
                    }
                }
            }

        } else {
            panic!("Depth cannot be zero");
        }

    } else {
        panic!{"Game ended, cannot make a move!"};
    }
}



fn eval(game: &reversi::Game, depth: u8) -> (i16, EvalReturn) {

    match game.get_status() {
        reversi::Status::Ended => (game.get_score_diff(), EvalReturn::Ended),
        reversi::Status::Running { current_player } => {
            if depth == 0 {
                match current_player {
                    reversi::Player::Light => (heavy_eval(&game) + BONUS_TURN, EvalReturn::Running { fixed_cells_board: quick_stable_board(&game.get_board(), reversi::Player::Light) }),
                    reversi::Player::Dark  => (heavy_eval(&game) - BONUS_TURN, EvalReturn::Running { fixed_cells_board: quick_stable_board(&game.get_board(), reversi::Player::Dark) }),
                }
            } else {

                match current_player {

                    reversi::Player::Light => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = LIGHT_STARTING_SCORE;
                        let mut best_end_score: i16 = LIGHT_STARTING_SCORE;
                        let mut fixed_cells_boards: Vec<reversi::Board> = Vec::new();
                        let mut game_after_move = game.clone();

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_eval_return) = eval(&game_after_move, depth - 1);

                                    if let EvalReturn::Ended = current_eval_return {
                                        if current_score > best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else if let EvalReturn::Running { fixed_cells_board: current_fixed_cells_board} = current_eval_return {
                                        num_moves += 1;
                                        end_game = false;
                                        fixed_cells_boards.push(current_fixed_cells_board);
                                        if current_score > best_score {
                                            best_score = current_score;
                                        }
                                    }

                                    game_after_move = game.clone();

                                }
                            }
                        }

                        if end_game || best_end_score > 0 || (best_end_score == 0 && best_score < 0) {
                            (best_end_score, EvalReturn::Ended)
                        } else {
                            let (fixed_cells_board, score_diff) = board_intersec(&mut fixed_cells_boards);
                            (best_score + MOBILITY*num_moves + score_diff, EvalReturn::Running { fixed_cells_board: fixed_cells_board })
                        }
                    }

                    reversi::Player::Dark  => {
                        let mut end_game: bool = true;
                        let mut num_moves: i16 = 0;
                        let mut best_score = DARK_STARTING_SCORE;
                        let mut best_end_score: i16 = DARK_STARTING_SCORE;
                        let mut fixed_cells_boards: Vec<reversi::Board> = Vec::new();
                        let mut game_after_move = game.clone();

                        for row in 0..reversi::BOARD_SIZE {
                            for col in 0..reversi::BOARD_SIZE {
                                if game_after_move.make_move((row, col)) {

                                    let (current_score, current_eval_return) = eval(&game_after_move, depth - 1);

                                    if let EvalReturn::Ended = current_eval_return {
                                        if current_score < best_end_score {
                                            best_end_score = current_score;
                                        }
                                    } else if let EvalReturn::Running { fixed_cells_board: current_fixed_cells_board} = current_eval_return {
                                        num_moves += 1;
                                        end_game = false;
                                        fixed_cells_boards.push(current_fixed_cells_board);
                                        if current_score < best_score {
                                            best_score = current_score;
                                        }
                                    }

                                    game_after_move = game.clone();

                                }
                            }
                        }

                        if end_game || best_end_score < 0 || (best_end_score == 0 && best_score > 0) {
                            (best_end_score, EvalReturn::Ended)
                        } else {
                            let (fixed_cells_board, score_diff) = board_intersec(&mut fixed_cells_boards);
                            (best_score - MOBILITY*num_moves + score_diff, EvalReturn::Running { fixed_cells_board: fixed_cells_board })
                        }
                    }

                }

            }
        }
    }
}


fn board_intersec(boards: &mut Vec<reversi::Board>) -> (reversi::Board, i16) {

    let mut intersection_board: reversi::Board = [[reversi::Cell::Empty; reversi::BOARD_SIZE]; reversi::BOARD_SIZE];

    if let Some(first_board) = boards.pop() {

        let mut score_diff: i16 = 0;
        for (row, row_array) in first_board.iter().enumerate() {
            'cell_loop: for (col, cell) in row_array.iter().enumerate() {
                match *cell {
                    reversi::Cell::Taken { disk: reversi::Player::Light } => {
                        for next_board in boards.iter() {
                            if let reversi::Cell::Taken { disk: reversi::Player::Light } = next_board[row][col] {
                                continue;
                            } else {
                                continue 'cell_loop;
                            }
                        }
                        intersection_board[row][col] = reversi::Cell::Taken { disk: reversi::Player::Light };
                        score_diff += 1;
                    }
                    reversi::Cell::Taken { disk: reversi::Player::Dark } => {
                        for next_board in boards.iter() {
                            if let reversi::Cell::Taken { disk: reversi::Player::Dark } = next_board[row][col] {
                                continue;
                            } else {
                                continue 'cell_loop;
                            }
                        }
                        intersection_board[row][col] = reversi::Cell::Taken { disk: reversi::Player::Dark };
                        score_diff -= 1;
                    }
                    _ => {}
                }
            }
        }

        return (intersection_board, score_diff);

    } else {
        return (intersection_board, 0);
    }

}



fn quick_stable_board(board: &reversi::Board, player: reversi::Player) -> reversi::Board {

    let mut stable_board = board.clone();

    for row in 0..reversi::BOARD_SIZE {
        for col in 0..reversi::BOARD_SIZE {
            if let reversi::Cell::Empty = board[row][col] {

                for &(delta_ns, delta_ew) in reversi::DIRECTIONS.iter() {
                    if check_move_along_direction(player, board, (row, col), (delta_ns, delta_ew)) {

                        stable_board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] = reversi::Cell::Empty;

                        let (mut row_i8, mut col_i8): (i8, i8) = (row as i8 + 2*delta_ns, col as i8 + 2*delta_ew);

                        while let reversi::Cell::Taken { disk } = board[ row_i8 as usize ][ col_i8 as usize ] {
                            if player == disk {
                                break;
                            }

                            stable_board[ row_i8 as usize ][ col_i8 as usize ] = reversi::Cell::Empty;

                            row_i8 += delta_ns;
                            col_i8 += delta_ew;
                        }
                    }
                }

            }
        }
    }

    return stable_board;
}



fn check_move_along_direction (current_player: reversi::Player, board: &reversi::Board, (row, col): (usize, usize), (delta_ns, delta_ew): (i8, i8)) -> bool {

    // Need at least two cells' space in the given direction
    let mut col_i8: i8 = col as i8 + 2*delta_ew;
    if ( col_i8 < 0 ) || ( col_i8 >= reversi::BOARD_SIZE as i8 ) {
            return false;
    }

    let mut row_i8: i8 = row as i8 + 2*delta_ns;
    if ( row_i8 < 0 ) || ( row_i8 >= reversi::BOARD_SIZE as i8 ) {
            return false;
    }

    // Next cell has to be owned by the other player
    if let reversi::Cell::Taken { disk } = board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] {
        if disk == current_player {
            return false;
        }

        while let reversi::Cell::Taken { disk } = board[ row_i8 as usize ][ col_i8 as usize] {
            if disk == current_player {
                return true;
            }

            col_i8 += delta_ew;
            if ( col_i8 < 0 ) || ( col_i8 >= reversi::BOARD_SIZE as i8 ) {
                    return false;
            }

            row_i8 += delta_ns;
            if ( row_i8 < 0 ) || ( row_i8 >= reversi::BOARD_SIZE as i8 ) {
                    return false;
            }

        }
    }

    false
}



fn heavy_eval(game: &reversi::Game) -> i16 {
    const CORNER_BONUS: i16 = 15;
    const ODD_MALUS: i16 = 3;
    const EVEN_BONUS: i16 = 3;
    const ODD_CORNER_MALUS: i16 = 10;
    const EVEN_CORNER_BONUS: i16 = 5;
    const FIXED_BONUS: i16 = 3;

    const SIDES: [( (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize), (usize, usize) ); 4] = [
        ( (0,0), (0,1), (1,1), (0,2), (2,2), (1,0), (2,0) ), // NW corner
        ( (0,7), (1,7), (1,6), (2,7), (2,5), (0,6), (0,5) ), // NE corner
        ( (7,0), (6,0), (6,1), (5,0), (5,2), (7,1), (7,2) ), // SW corner
        ( (7,7), (6,7), (6,6), (5,7), (5,5), (7,6), (7,6) ), // SE corner
        ];


    //let (score_light, score_dark) = game.get_score();
    let mut score: i16 = 0; // ( game.get_score_diff() * FIXED_BONUS * game.get_tempo() as i16 ) / 64; // (score_light as i16) - (score_dark as i16);

    for &(corner, odd, odd_corner, even, even_corner, counter_odd, counter_even) in SIDES.iter() {

        if let reversi::Cell::Taken { disk } = game.get_cell(corner) {
            match disk {
                reversi::Player::Light => {
                    score += CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(even) {
                            score += FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(counter_odd) {
                        score += FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Light } = game.get_cell(counter_even) {
                            score += FIXED_BONUS;
                        }
                    }
                }
                reversi::Player::Dark => {
                    score -= CORNER_BONUS;
                    if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(even) {
                            score -= FIXED_BONUS;
                        }
                    }
                    if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(counter_odd) {
                        score -= FIXED_BONUS;
                        if let reversi::Cell::Taken { disk: reversi::Player::Dark } = game.get_cell(counter_even) {
                            score -= FIXED_BONUS;
                        }
                    }
                }
            }

        } else {

            if let reversi::Cell::Taken { disk } = game.get_cell(odd) {
                match disk {
                    reversi::Player::Light => score -= ODD_MALUS,
                    reversi::Player::Dark  => score += ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(even) {
                match disk {
                    reversi::Player::Light => score += EVEN_BONUS,
                    reversi::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(counter_odd) {
                match disk {
                    reversi::Player::Light => score -= ODD_MALUS,
                    reversi::Player::Dark  => score += ODD_MALUS,
                }
            } else if let reversi::Cell::Taken { disk } = game.get_cell(counter_even) {
                match disk {
                    reversi::Player::Light => score += EVEN_BONUS,
                    reversi::Player::Dark  => score -= EVEN_BONUS,
                }
            }

            if let reversi::Cell::Taken { disk } = game.get_cell(odd_corner) {
                match disk {
                    reversi::Player::Light => score -= ODD_CORNER_MALUS,
                    reversi::Player::Dark  => score += ODD_CORNER_MALUS,
                }

            } else if let reversi::Cell::Taken { disk } = game.get_cell(even_corner) {
                match disk {
                    reversi::Player::Light => score += EVEN_CORNER_BONUS,
                    reversi::Player::Dark  => score -= EVEN_CORNER_BONUS,
                }
            }
        }
    }

    score
}
