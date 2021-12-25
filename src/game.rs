use crate::board::Board;
use crate::cells::Position;
use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::errors::InvalidMove;
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

        let edges = Game::get_edges(self.current_player);

        if self.board.is_in_same_set(edges.0, edges.1) {
            self.status = Status::Finished(self.current_player);
        } else {
            self.current_player = self.current_player.opponent_color();
        }

        Ok(())
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
}
