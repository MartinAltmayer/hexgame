pub use crate::square_array::Coords;
use crate::square_array::SquareArray;
use std::cmp::Ordering;
use std::error;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    BLACK,
    WHITE,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    TOP,
    RIGHT,
    BOTTOM,
    LEFT,
    Index(u16),
}

#[derive(Copy, Clone, Default)]
struct Cell {
    color: Option<Color>,
    parent: Option<Position>,
}

pub struct Board {
    cells: SquareArray<Cell>,
    right_parent: Option<Position>,
    bottom_parent: Option<Position>,
}

impl Board {
    pub fn new(size: u8) -> Board {
        // Neighbor calculations assume size >= 2
        assert!(size >= 2, "Size must be at least 2");
        Board {
            cells: SquareArray::new(size),
            right_parent: None,
            bottom_parent: None,
        }
    }

    pub fn size(&self) -> u8 {
        self.cells.size
    }

    pub fn get_color(&self, coords: Coords) -> Option<Color> {
        self.cells.at_coord(coords).color
    }

    fn get_color_of_position(&self, position: Position) -> Option<Color> {
        match position {
            Position::TOP | Position::BOTTOM => Some(Color::BLACK),
            Position::LEFT | Position::RIGHT => Some(Color::WHITE),
            Position::Index(index) => self.cells.at_index(index).color,
        }
    }

    fn get_neighbors(&self, index: u16) -> Vec<Position> {
        let mut neighbors = Vec::new();
        let size: u16 = self.size().into();

        if index % size == 0 {
            neighbors.push(Position::LEFT);
        } else {
            neighbors.push(Position::Index(index - 1));
        }

        if index % size == size - 1 {
            neighbors.push(Position::RIGHT);
        } else {
            neighbors.push(Position::Index(index + 1));
        }

        if index < size {
            neighbors.push(Position::TOP);
        } else {
            neighbors.push(Position::Index(index - size));
            if index % size < size - 1 {
                neighbors.push(Position::Index(index - size + 1));
            }
        }

        if index >= size * (size - 1) {
            neighbors.push(Position::BOTTOM);
        } else {
            neighbors.push(Position::Index(index + size));
            if index % size > 0 {
                neighbors.push(Position::Index(index + size - 1))
            }
        }

        neighbors
    }

    fn get_parent(&self, position: Position) -> Option<Position> {
        match position {
            Position::TOP | Position::LEFT => None,
            Position::BOTTOM => self.bottom_parent,
            Position::RIGHT => self.right_parent,
            Position::Index(index) => self.cells.at_index(index).parent,
        }
    }

    fn set_parent(&mut self, position: Position, parent: Position) {
        match position {
            Position::TOP | Position::LEFT => panic!("Cannot set parent of {:?}", position),
            Position::BOTTOM => {
                self.bottom_parent = Some(parent);
            }
            Position::RIGHT => {
                self.right_parent = Some(parent);
            }
            Position::Index(index) => {
                self.cells.at_index(index).parent = Some(parent);
            }
        }
    }

    pub fn find_root(&mut self, position: Position) -> Position {
        let mut root = position;
        loop {
            match self.get_parent(root) {
                Some(p) => {
                    root = p;
                }
                None => break,
            }
        }

        loop {
            let parent = self.get_parent(position);
            if parent != Some(root) {
                self.set_parent(position, root)
            } else {
                break;
            }
        }

        root
    }

    fn compare(&self, index: u16, position: Position) -> Ordering {
        match position {
            Position::TOP | Position::LEFT => Ordering::Greater,
            Position::BOTTOM | Position::RIGHT => Ordering::Less,
            Position::Index(pos_index) => index.cmp(&pos_index),
        }
    }

    pub fn merge(&mut self, index: u16, position: Position) {
        let root = self.find_root(position);
        match self.compare(index.parent, root) {
            Ordering::Greater => {
                self.set_parent(Position::Index(index), root);
            }
            Ordering::Less => {
                self.set_parent(position, Position::Index(index));
            }
            Ordering::Equal => (),
        }
    }

    pub fn is_connected(&mut self, position1: Position, position2: Position) -> bool {
        self.find_root(position1) == self.find_root(position2)
    }

    pub fn play(&mut self, coords: Coords, color: Color) -> Result<(), InvalidMove> {
        if coords.row >= self.size() || coords.column >= self.size() {
            return Err(InvalidMove::OutOfBounds(coords));
        }

        let index = self.cells.index_from_coords(coords);
        match self.cells.at_index(index).color {
            None => {
                self.cells.set_index(
                    index,
                    Cell {
                        color: Some(color),
                        parent: None,
                    },
                );
                for neighbor in self.get_neighbors(index) {
                    if self.get_color_of_position(neighbor) == Some(color) {
                        self.merge(index, neighbor);
                    }
                }
                Ok(())
            }
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

impl error::Error for InvalidMove {}

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

#[cfg(test)]
mod test_neighbors {
    use super::*;
    use std::collections::HashSet;
    use std::iter::FromIterator;

    fn check_neighbors(board: &Board, row: u8, column: u8, expected: &[Position]) {
        let index = board.cells.index_from_coords(Coords { row, column });
        let neighbors = HashSet::<Position>::from_iter(board.get_neighbors(index).iter().copied());
        let expected = HashSet::from_iter(expected.iter().copied());
        assert_eq!(neighbors, expected);
    }

    fn make_position(board: &Board, row: u8, column: u8) -> Position {
        Position::Index(board.cells.index_from_coords(Coords { row, column }))
    }

    #[test]
    fn test_neighbors_top_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 0, &[
            Position::TOP,
            Position::LEFT,
            make_position(&board, 0, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 1, &[
            Position::TOP,
            make_position(&board, 0, 0),
            make_position(&board, 0, 2),
            make_position(&board, 1, 0),
            make_position(&board, 1, 1),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 0, &[
            Position::LEFT,
            make_position(&board, 0, 0),
            make_position(&board, 0, 1),
            make_position(&board, 1, 1),
            make_position(&board, 2, 0),
        ]);
    }

    #[test]
    fn test_neighbors_top_right_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 4, &[
            Position::TOP,
            Position::RIGHT,
            make_position(&board, 0, 3),
            make_position(&board, 1, 3),
            make_position(&board, 1, 4),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 3, &[
            Position::TOP,
            make_position(&board, 0, 2),
            make_position(&board, 0, 4),
            make_position(&board, 1, 2),
            make_position(&board, 1, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 4, &[
            Position::RIGHT,
            make_position(&board, 0, 4),
            make_position(&board, 1, 3),
            make_position(&board, 2, 3),
            make_position(&board, 2, 4),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 0, &[
            Position::BOTTOM,
            Position::LEFT,
            make_position(&board, 3, 0),
            make_position(&board, 3, 1),
            make_position(&board, 4, 1),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 0, &[
            Position::LEFT,
            make_position(&board, 2, 0),
            make_position(&board, 2, 1),
            make_position(&board, 3, 1),
            make_position(&board, 4, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 1, &[
            Position::BOTTOM,
            make_position(&board, 3, 1),
            make_position(&board, 3, 2),
            make_position(&board, 4, 0),
            make_position(&board, 4, 2),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 4, &[
            Position::BOTTOM,
            Position::RIGHT,
            make_position(&board, 3, 4),
            make_position(&board, 4, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 4, &[
            Position::RIGHT,
            make_position(&board, 2, 4),
            make_position(&board, 3, 3),
            make_position(&board, 4, 3),
            make_position(&board, 4, 4),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 3, &[
            Position::BOTTOM,
            make_position(&board, 3, 3),
            make_position(&board, 3, 4),
            make_position(&board, 4, 2),
            make_position(&board, 4, 4),
        ]);
    }

    #[test]
    fn test_neighbors_center() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 2, 2, &[
            make_position(&board, 1, 2),
            make_position(&board, 1, 3),
            make_position(&board, 2, 1),
            make_position(&board, 2, 3),
            make_position(&board, 3, 1),
            make_position(&board, 3, 2),
        ]);
    }
}
