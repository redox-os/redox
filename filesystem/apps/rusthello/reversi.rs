//! It provides the main structures and mechanics for a Reversi game.

/// There are two players playing the match: Light and Dark
#[derive(Clone, Copy, PartialEq)]
pub enum Disk {
    Light,
    Dark,
}

impl Disk {
    /// Get self's opposite side
    pub fn opposite(&self) -> Disk {
        match *self {
            Disk::Light => Disk::Dark,
            Disk::Dark => Disk::Light,
        }
    }
}



/// A game can be in two status: either running (with a next player to play) or ended.
#[derive(Clone)]
pub enum Status {
    Running { current_turn: Disk },
    Ended,
}



/// Each cell in the board can either be empty or taken by one of the players.
#[derive(Clone, Copy)]
pub enum Cell {
    Taken { disk: Disk },
    Empty,
}



/// An array listing all the cardinal directions, represented by the coordinate delta to move in that direction.
/// #Examples
/// If I am in cell (4, 5) and move NE, I go to cell (4, 5) + (1, -1) = (5, 4).
pub const DIRECTIONS: [(i8, i8); 8] = [
    ( 1,  0), //North
    ( 1,  1), //NE
    ( 0,  1), //East
    (-1,  1), //SE
    (-1,  0), //South
    (-1, -1), //SW
    ( 0, -1), //West
    ( 1, -1), //NW
    ];

/// The size of the board is a constant.
pub const BOARD_SIZE: usize = 8;

/// Board is the type of boards, that is, bidimensional arrays of Cells of size BOARD_SIZE.
pub type Board = [[Cell; BOARD_SIZE]; BOARD_SIZE];



/// The board is given by a matrix of cells of size BOARD_SIZE and by which player has to move next.
#[derive(Clone)]
pub struct Game {
    board: Board,
    status: Status,
    score_light: u8,
    score_dark: u8,
}

impl Game {
    /// Initializing a new game: starting positions on the board and Dark is the first to play
    pub fn new() -> Game {
        let mut board: Board = [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = Cell::Taken { disk: Disk::Light };
        board[4][4] = Cell::Taken { disk: Disk::Light };
        board[3][4] = Cell::Taken { disk: Disk::Dark };
        board[4][3] = Cell::Taken { disk: Disk::Dark };

        Game {
            board: board,
            status: Status::Running { current_turn: Disk::Dark },
            score_light: 2,
            score_dark: 2,
        }
    }



    /// Returns the game's board
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Returns the game's status
    pub fn get_status(&self) -> Status {
        self.status.clone()
    }

    /// Returns the current score of the match.
    pub fn get_score(&self) -> (u8, u8) {
        (self.score_light, self.score_dark)
    }

    /// Returns the difference in score between Light and Dark.
    pub fn get_score_diff(&self) -> i16 {
        self.score_light as i16 - self.score_dark as i16
    }

    /// Returns game's tempo (how many disks there are on the board).
    pub fn get_tempo(&self) -> u8 {
        self.score_light + self.score_dark
    }

    /// Returns the board's cell corresponding to the given coordinates.
    pub fn get_cell(&self, (row, col): (usize, usize)) -> Cell {
        self.board[row][col]
    }



    /// Check that a given move is legal
    pub fn check_move (&self, (row, col): (usize, usize)) -> bool {

        // If the given coordinate falls out of the board or in a taken cell, the move cannot be legal
        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return false;
        } else if let Cell::Taken { .. } = self.board[row][col] {
            return false;
        }

        if let Status::Running { current_turn } = self.status {
            return self.quick_check_move(current_turn, (row, col));
        }

        false
    }



    /// Check that a given move is legal, but faster (by skipping unnecessary checks)
    fn quick_check_move (&self, current_turn: Disk, (row, col): (usize, usize)) -> bool {

        // If a move leads to eat in at least one direction, then it is legal
        for &dir in DIRECTIONS.iter() {
            if self.quick_check_move_along_direction(current_turn, (row, col), dir) {
                return true;
            }
        }

        false
    }



    /// Check whether a move leads to eat in a specified direction, but faster (by skipping unnecessary checks)
    /// Does NOT perform checks already performed by check_move!
    fn quick_check_move_along_direction (&self, current_turn: Disk, (row, col): (usize, usize), (delta_ns, delta_ew): (i8, i8)) -> bool {

        // Need at least two cells' space in the given direction
        let mut col_i8: i8 = col as i8 + 2*delta_ew;
        if ( col_i8 < 0 ) || ( col_i8 >= BOARD_SIZE as i8 ) {
                return false;
        }

        let mut row_i8: i8 = row as i8 + 2*delta_ns;
        if ( row_i8 < 0 ) || ( row_i8 >= BOARD_SIZE as i8 ) {
                return false;
        }

        // Next cell has to be owned by the other player
        if let Cell::Taken { disk } = self.board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] {
            if disk == current_turn {
                return false;
            }

            while let Some(&rows) = self.board.get(row_i8 as usize) {
                if let Some(&cell) = rows.get(col_i8 as usize) {
                    if let Cell::Taken { disk } = cell {
                        if disk == current_turn {
                            return true;
                        }
                        col_i8 += delta_ew;
                        row_i8 += delta_ns;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }

        false
    }




    /// Eats all of the opponent's occupied cells from a specified cell (given by its coordinates) in a specified direction
    /// until it finds a cell of the current player
    /// WARNING: this function do NOT perform any check about whether or not the move is legal
    fn eat_along_direction (&mut self, current_turn: Disk, (row, col): (usize, usize), (delta_ns, delta_ew): (i8, i8)) {

        self.board[ ( row as i8 + delta_ns ) as usize ][ ( col as i8 + delta_ew ) as usize] = Cell::Taken { disk: current_turn };

        let (mut row_i8, mut col_i8): (i8, i8) = (row as i8 + 2*delta_ns, col as i8 + 2*delta_ew);

        let mut eating: u8 = 1;

        while let Some(rows) = self.board.get_mut(row_i8 as usize) {
            if let Some(cell) = rows.get_mut(col_i8 as usize) {
                if let Cell::Taken { disk } = *cell {
                    if current_turn == disk {
                        break;
                    } else {
                        *cell = Cell::Taken { disk: current_turn };
                        eating += 1;
                        row_i8 += delta_ns;
                        col_i8 += delta_ew;
                    }
                }
            }
        }

        match current_turn {
            Disk::Light => {
                self.score_light += eating;
                self.score_dark  -= eating;
            }
            Disk::Dark => {
                self.score_light -= eating;
                self.score_dark  += eating;
            }
        }
    }


    /// Current player perform a move, after verifying that it is legal.
    /// It returns whether the move is legal or not.
    pub fn make_move (&mut self, (row, col): (usize, usize)) -> bool {

        if row >= BOARD_SIZE || col >= BOARD_SIZE {
            return false;
        } else if let Cell::Taken { .. } = self.board[row][col] {
            return false;
        }

        let mut legal: bool = false;

        if let Status::Running { current_turn } = self.status {
            for &dir in DIRECTIONS.iter() {
                if self.quick_check_move_along_direction(current_turn, (row, col), dir) {
                    self.eat_along_direction(current_turn, (row, col), dir);
                    legal = true;
                }
            }
        }

        // If a move is legal, the next player to play has to be determined
        // If the opposite player can make any move at all, it gets the turn
        // If not, if the previous player can make any move at all, it gets the turn
        // If not (that is, if no player can make any move at all) the game is ended
        if legal {
            if let Status::Running { current_turn } = self.status {
                self.board[row][col] = Cell::Taken { disk: current_turn };
                match current_turn {
                    Disk::Light => self.score_light += 1,
                    Disk::Dark =>  self.score_dark  += 1,
                }

                if self.get_tempo() == BOARD_SIZE as u8 * BOARD_SIZE as u8 {
                    self.status = Status::Ended;
                } else {
                    self.status = Status::Running { current_turn: current_turn.opposite() };
                    //self.flip_game_status();
                    if !self.can_move() {
                        self.status = Status::Running { current_turn: current_turn };
                        //self.flip_game_status();
                        if !self.can_move() {
                            self.status = Status::Ended;
                        }
                    }
                }
            }
        }

        legal
    }

    /// Returns whether or not next_player can make any move at all.
    fn can_move(&self) -> bool {
        if let Status::Running{ current_turn } = self.status {
            for (row_n, row) in self.board.iter().enumerate() {
                for (col_n, &cell) in row.iter().enumerate() {
                    if let Cell::Empty = cell {
                        if self.quick_check_move(current_turn, (row_n, col_n)) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

}
