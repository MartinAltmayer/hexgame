use crate::board::{Board, Color, Coords, InvalidMove, Position};
use crate::union_find::UnionFind;

#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Ongoing,
    Finished(Color),
}

pub struct Game {
    pub board: Board,
    pub current_player: Color,
    pub status: Status,
}

impl Game {
    pub fn new(size: u8) -> Game {
        Game {
            board: Board::new(size),
            current_player: Color::BLACK,
            status: Status::Ongoing,
        }
    }

    pub fn get_edges(color: Color) -> (Position, Position) {
        match color {
            Color::BLACK => (Position::TOP, Position::BOTTOM),
            Color::WHITE => (Position::LEFT, Position::RIGHT),
        }
    }

    pub fn play(&mut self, coords: Coords) -> Result<(), InvalidMove> {
        self.board.play(coords, self.current_player)?;

        let edges = Game::get_edges(self.current_player);

        if self.board.is_in_same_set(edges.0, edges.1) {
            self.status = Status::Finished(self.current_player);
        } else {
            self.current_player = match self.current_player {
                Color::BLACK => Color::WHITE,
                Color::WHITE => Color::BLACK,
            };
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
        assert_eq!(game.board.get_color(coord1).unwrap(), Color::BLACK);
        assert_eq!(game.board.get_color(coord2).unwrap(), Color::WHITE);
        assert_eq!(game.current_player, Color::BLACK);
        assert_eq!(game.status, Status::Ongoing);
    }
}
