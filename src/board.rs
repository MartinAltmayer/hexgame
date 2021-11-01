pub use crate::square_array::Coords;
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

    pub fn get_color(&self, coords: Coords) -> Option<Color> {
        self.cells.at_coord(coords).color
    }

    pub fn play(&mut self, coords: Coords, color: Color) -> Result<(), InvalidMove> {
        if coords.row >= self.size() || coords.column >= self.size() {
            return Err(InvalidMove::OutOfBounds(coords));
        }

        let index = self.cells.index_from_coord(coords);
        match self.cells.at_index(index).color {
            None => Ok(self.cells.set_index(index, Cell { color: Some(color) })),
            _ => Err(InvalidMove::CellOccupied(coords)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum InvalidMove {
    OutOfBounds(Coords),
    CellOccupied(Coords),
}

impl fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            InvalidMove::OutOfBounds(coords) => {
                write!(f, "Coordinates {} are out of bounds", coords)
            }
            InvalidMove::CellOccupied(coords) => {
                write!(f, "Cell {} is already occupied", coords)
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
        assert!(board.get_color(Coords { row: 0, column: 0 }).is_none());
    }

    #[test]
    fn test_play() {
        let mut board = Board::new(3);
        let coords = Coords { row: 1, column: 2 };
        let result = board.play(coords, Color::BLACK);
        assert!(result.is_ok());
        assert_eq!(board.get_color(coords).unwrap(), Color::BLACK);
    }

    #[test]
    fn test_play_row_out_of_bounds() {
        let mut board = Board::new(3);
        let coords = Coords { row: 3, column: 2 };
        let error = board.play(coords, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds(coords));
    }

    #[test]
    fn test_play_column_out_of_bounds() {
        let mut board = Board::new(3);
        let coords = Coords { row: 0, column: 3 };
        let error = board.play(coords, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds(coords));
    }

    #[test]
    fn test_play_on_occupied_cell() {
        let mut board = Board::new(3);
        let coords = Coords { row: 1, column: 2 };
        let _ = board.play(coords, Color::BLACK);
        let error = board.play(coords, Color::BLACK).unwrap_err();
        assert_eq!(error, InvalidMove::CellOccupied(coords));
    }
}
