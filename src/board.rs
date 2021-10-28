use crate::square_array::SquareArray;

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

#[derive(Debug, PartialEq)]
struct CellOccupied {
    row: u8,
    column: u8,
}

impl Board {
    fn new(size: u8) -> Board {
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

    pub fn play(&mut self, row: u8, column: u8, color: Color) -> Result<(), CellOccupied> {
        let index = self.cells.index_from_coord(row, column);
        match self.cells.at_index(index).color {
            None => {
                self.cells.set_index(index, Cell { color: Some(color) });
                Ok(())
            }
            _ => Err(CellOccupied { row, column }),
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
    fn test_play_on_occupied_cell() {
        let mut board = Board::new(3);
        let _ = board.play(1, 2, Color::BLACK);
        let error = board.play(1, 2, Color::BLACK).unwrap_err();
        assert_eq!(error, CellOccupied { row: 1, column: 2 });
    }
}
