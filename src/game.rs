use crate::board::{Board, CellOccupied, Color};

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

    pub fn play(&mut self, row: u8, column: u8) -> Result<(), CellOccupied> {
        self.board.play(row, column, self.current_player)?;

        self.current_player = match self.current_player {
            Color::BLACK => Color::WHITE,
            Color::WHITE => Color::BLACK,
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_match() {
        let mut game = Game::new(3);
        game.play(1, 2).ok();
        game.play(2, 1).ok();
        assert_eq!(game.board.get_color(1, 2).unwrap(), Color::BLACK);
        assert_eq!(game.board.get_color(2, 1).unwrap(), Color::WHITE);
        assert_eq!(game.current_player, Color::BLACK);
        assert_eq!(game.status, Status::Ongoing);
    }
}
