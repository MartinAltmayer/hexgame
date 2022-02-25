use crate::board::{Board, StoneMatrix};
use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::edges::get_edges_of_color;
use crate::errors::{InvalidBoard, InvalidMove};

/// Status of a game.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Game has not yet ended. The `Color` indicates which player is to make the next turn.
    Ongoing(Color),
    /// Game has been finished. The `Color` indicates which player has won the game.
    /// Note that a game of Hex never ends in a draw.
    Finished(Color),
}

/// `Game` holds the full state of a game of Hex and allows to manipulate this state by playing valid moves.
///
/// The game state consists of a board (`get_board`) and the current player (`get_current_player`).
#[derive(Clone)]
pub struct Game {
    board: Board,
    status: Status,
}

impl Game {
    /// Create a new game with the given board size. Boards are always square.
    ///
    /// Games always start with black.
    ///
    /// This method will panic if the size is not bounded by `MIN_BOARD_SIZE` and `MAX_BOARD_SIZE`.
    pub fn new(size: CoordValue) -> Game {
        Game {
            board: Board::new(size),
            status: Status::Ongoing(Color::Black),
        }
    }

    /// Return the game's board.
    pub fn get_board(&self) -> &Board {
        &self.board
    }

    /// Return the current player, or None if the game has ended.
    pub fn get_current_player(&self) -> Option<Color> {
        match self.status {
            Status::Ongoing(color) => Some(color),
            Status::Finished(_) => None,
        }
    }

    // Get the game's status.
    pub fn get_status(&self) -> Status {
        self.status
    }

    /// Load a game from a `StoneMatrix` and a current player color.
    ///
    /// This method returns an error if `current_player` is None, but the game has not yet finished.
    /// Conversely, if the game has finished, `current_player` will be ignored.
    ///
    /// Please also have a look at the `Serialization` trait which allows to directly deserialize a game from JSON.
    pub fn load(stones: StoneMatrix, current_player: Option<Color>) -> Result<Self, InvalidBoard> {
        let mut board = Board::from_stone_matrix(stones)?;
        let status = Self::compute_status(&mut board, current_player)?;
        Ok(Self { board, status })
    }

    /// Let the current player place a stone at the given coordinates.
    /// If the move is invalid, this method returns an error.
    /// This method will automatically update the current player.
    pub fn play(&mut self, coords: Coords) -> Result<(), InvalidMove> {
        match self.status {
            Status::Ongoing(current_player) => {
                self.board.play(coords, current_player)?;

                if Self::is_finished_after_player(&self.board, current_player) {
                    self.status = Status::Finished(current_player);
                } else {
                    self.status = Status::Ongoing(current_player.opponent_color());
                }

                Ok(())
            }
            Status::Finished(_) => Err(InvalidMove::GameOver),
        }
    }

    fn compute_status(
        board: &mut Board,
        current_player: Option<Color>,
    ) -> Result<Status, InvalidBoard> {
        if Self::is_finished_after_player(board, Color::Black) {
            return Ok(Status::Finished(Color::Black));
        } else if Self::is_finished_after_player(board, Color::White) {
            return Ok(Status::Finished(Color::White));
        }

        match current_player {
            Some(color) => Ok(Status::Ongoing(color)),
            None => Err(InvalidBoard::NoCurrentPlayer),
        }
    }

    fn is_finished_after_player(board: &Board, current_player: Color) -> bool {
        let edges = get_edges_of_color(current_player);
        board.is_in_same_set(edges[0], edges[1])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match() {
        let mut game = Game::new(3);
        let coord1 = Coords { row: 1, column: 2 };
        let coord2 = Coords { row: 2, column: 1 };
        game.play(coord1).ok();
        game.play(coord2).ok();
        assert_eq!(game.board.get_color(coord1).unwrap(), Color::Black);
        assert_eq!(game.board.get_color(coord2).unwrap(), Color::White);
        assert_eq!(game.status, Status::Ongoing(Color::Black));
    }

    #[test]
    fn test_cannot_make_move_after_game_has_finished() {
        let mut game = Game::new(2);
        let _ = game.play(Coords { row: 0, column: 0 });
        let _ = game.play(Coords { row: 0, column: 1 });
        let _ = game.play(Coords { row: 1, column: 0 });
        let result = game.play(Coords { row: 1, column: 1 });

        assert_eq!(result, Err(InvalidMove::GameOver));
    }

    #[test]
    fn test_only_black_wins_on_vertical_connection() {
        let mut game = Game::new(3);
        let _ = game.play(Coords { row: 2, column: 2 }); // unused, let white start filling columns

        let _ = game.play(Coords { row: 0, column: 0 });
        let _ = game.play(Coords { row: 0, column: 1 });
        let _ = game.play(Coords { row: 1, column: 0 });
        let _ = game.play(Coords { row: 1, column: 1 });
        let _ = game.play(Coords { row: 2, column: 0 }); // white's vertical connection complete
        let _ = game.play(Coords { row: 2, column: 1 }); // black wins here

        assert_eq!(game.status, Status::Finished(Color::Black));
    }

    #[test]
    fn test_only_white_wins_on_horizontal_connection() {
        let mut game = Game::new(3);
        let _ = game.play(Coords { row: 0, column: 0 });
        let _ = game.play(Coords { row: 1, column: 0 });
        let _ = game.play(Coords { row: 0, column: 1 });
        let _ = game.play(Coords { row: 1, column: 1 });
        let _ = game.play(Coords { row: 0, column: 2 }); // black's horizontal connection complete
        let _ = game.play(Coords { row: 1, column: 2 }); // white wins here

        assert_eq!(game.status, Status::Finished(Color::White));
    }

    #[test]
    fn test_load_game() {
        let current_player = Color::Black;
        let stone_matrix = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None],
        ];
        let game = Game::load(stone_matrix, Some(current_player)).unwrap();

        assert_eq!(game.board.get_color(Coords::new(1, 0)), Some(Color::White));
        assert_eq!(game.status, Status::Ongoing(current_player));
    }

    #[test]
    fn test_load_game_without_current_player() {
        let stone_matrix = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None],
        ];
        let result = Game::load(stone_matrix, None);

        assert_eq!(result.err().unwrap(), InvalidBoard::NoCurrentPlayer);
    }

    #[test]
    fn test_load_finished_game() {
        let current_player = Color::Black;
        let stone_matrix = vec![
            vec![Some(Color::Black), Some(Color::White)],
            vec![Some(Color::White), Some(Color::Black)],
        ];
        let game = Game::load(stone_matrix, Some(current_player)).unwrap();

        assert_eq!(game.status, Status::Finished(Color::White));
        assert_eq!(game.get_current_player(), None);
    }
}
