use crate::board::{Board, StoneMatrix};
use crate::cells::Position;
use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::errors::{InvalidBoard, InvalidMove};
use crate::union_find::UnionFind;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    Ongoing,
    Finished(Color),
}

#[derive(Clone)]
pub struct Game {
    pub board: Board,
    pub current_player: Color,
    pub status: Status,
}

impl Game {
    pub fn new(size: CoordValue) -> Game {
        Game {
            board: Board::new(size),
            current_player: Color::Black,
            status: Status::Ongoing,
        }
    }

    pub fn load(stones: StoneMatrix, current_player: Color) -> Result<Self, InvalidBoard> {
        let mut board = Board::from_stone_matrix(stones)?;
        let status = Self::get_status(&mut board);
        Ok(Self {
            board,
            current_player,
            status,
        })
    }

    pub fn get_edges(color: Color) -> (Position, Position) {
        match color {
            Color::Black => (Position::Top, Position::Bottom),
            Color::White => (Position::Left, Position::Right),
        }
    }

    pub fn play(&mut self, coords: Coords) -> Result<(), InvalidMove> {
        if let Status::Finished(_) = self.status {
            return Err(InvalidMove::GameOver);
        }

        self.board.play(coords, self.current_player)?;

        if Self::is_finished_after_player(&mut self.board, self.current_player) {
            self.status = Status::Finished(self.current_player);
        } else {
            self.current_player = self.current_player.opponent_color();
        }

        Ok(())
    }

    fn get_status(board: &mut Board) -> Status {
        if Self::is_finished_after_player(board, Color::Black) {
            return Status::Finished(Color::Black);
        } else if Self::is_finished_after_player(board, Color::White) {
            return Status::Finished(Color::White);
        }
        Status::Ongoing
    }

    fn is_finished_after_player(board: &mut Board, current_player: Color) -> bool {
        let edges = Self::get_edges(current_player);
        board.is_in_same_set(edges.0, edges.1)
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
        assert_eq!(game.current_player, Color::Black);
        assert_eq!(game.status, Status::Ongoing);
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
        let game = Game::load(stone_matrix, current_player).unwrap();

        assert_eq!(game.board.get_color(Coords::new(1, 0)), Some(Color::White));
        assert_eq!(game.current_player, current_player);
        assert_eq!(game.status, Status::Ongoing);
    }

    #[test]
    fn test_load_finished_game() {
        let current_player = Color::Black;
        let stone_matrix = vec![
            vec![Some(Color::Black), Some(Color::White)],
            vec![Some(Color::White), Some(Color::Black)],
        ];
        let game = Game::load(stone_matrix, current_player).unwrap();

        assert_eq!(game.current_player, current_player);
        assert_eq!(game.status, Status::Finished(Color::White));
    }
}
