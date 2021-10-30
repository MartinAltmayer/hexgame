use crate::square_array::SquareArray;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    BLACK,
    WHITE,
}

#[derive(Copy, Clone, Default)]
pub struct Cell {
    color: Option<Color>,
}

pub struct Board {
    cells: SquareArray<Cell>,
}

impl Board {
    pub fn new(size: u8) -> Board {
        Board {
            cells: SquareArray::new(size),
        }
    }

    pub fn size(&self) -> u8 {
        self.cells.size
    }

    pub fn get_color(&self, row: u8, column: u8) -> Option<Color> {
        self.cells.at_coord(row, column).color
    }

    pub fn play(&mut self, row: u8, column: u8, color: Color) -> Result<(), InvalidMove> {
        if row >= self.size() || column >= self.size() {
            return Err(InvalidMove::OutOfBounds { row, column });
        }

        let index = self.cells.index_from_coord(row, column);
        match self.cells.at_index(index).color {
            None => Ok(self.cells.set_index(index, Cell { color: Some(color) })),
            _ => Err(InvalidMove::CellOccupied { row, column }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InvalidMove {
    OutOfBounds { row: u8, column: u8 },
    CellOccupied { row: u8, column: u8 },
}

impl fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            InvalidMove::OutOfBounds { row, column } => {
                write!(f, "Coordinates ({}, {}) are out of bounds", row, column)
            }
            InvalidMove::CellOccupied { row, column } => {
                write!(f, "Cell ({}, {}) is already occupied", row, column)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let board = Board::new(3);
        assert_eq!(board.size(), 3);
        assert!(board.get_color(0, 0).is_none());
    }

    #[test]
    fn test_play() {
        let mut board = Board::new(3);
        let result = board.play(1, 2, Color::BLACK);
        assert!(result.is_ok());
        assert_eq!(board.get_color(1, 2).unwrap(), Color::BLACK);
    }

    #[test]
    fn test_play_row_out_of_bounds() {
        let mut board = Board::new(3);
        let error = board.play(3, 2, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds { row: 3, column: 2 });
    }

    #[test]
    fn test_play_column_out_of_bounds() {
        let mut board = Board::new(3);
        let error = board.play(0, 3, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds { row: 0, column: 3 });
    }

    #[test]
    fn test_play_on_occupied_cell() {
        let mut board = Board::new(3);
        let _ = board.play(1, 2, Color::BLACK);
        let error = board.play(1, 2, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::CellOccupied { row: 1, column: 2 });
    }
}
