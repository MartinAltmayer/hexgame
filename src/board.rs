use crate::coords::{CoordValue, Coords};
use crate::errors::{InvalidBoard, InvalidMove};
use crate::square_array::{Index, SquareArray};
use crate::union_find::UnionFind;
use std::iter;
use std::iter::Iterator;

// Neighbor calculations assume size >= 2
pub const MIN_BOARD_SIZE: CoordValue = 2;
// Technically, we support much larger boards, but future optimizations may restrict this.
pub const MAX_BOARD_SIZE: CoordValue = 19;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn opponent_color(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
    // Order is important here: UnionFind's merge always chooses larger positions as roots.
    // Thus, BOTTOM and RIGHT are always their own parent and we do not have to store their parents.
    Index(Index),
    Top,
    Left,
    Bottom,
    Right,
}

#[derive(Copy, Clone, Default)]
struct Cell {
    color: Option<Color>,
    parent: Option<Position>,
}

#[derive(Clone)]
pub struct Board {
    cells: SquareArray<Cell>,
    top_parent: Option<Position>,
    left_parent: Option<Position>,
}

impl Board {
    pub fn new(size: CoordValue) -> Self {
        check_board_size(size as usize).expect("Invalid size");
        Self {
            cells: SquareArray::new(size),
            top_parent: None,
            left_parent: None,
        }
    }

    pub fn from_cells(cells: Vec<Vec<Option<Color>>>) -> Result<Self, InvalidBoard> {
        let size = check_board_size(cells.len())?;
        let mut board = Self::new(size);

        for (row, cells_in_row) in (0..size).zip(cells) {
            if cells_in_row.len() != size as usize {
                return Err(InvalidBoard::NotSquare(row, size));
            }
            for (column, cell) in (0..size).zip(cells_in_row) {
                if let Some(color) = cell {
                    board.play(Coords { row, column }, color).unwrap();
                }
            }
        }
        Ok(board)
    }

    pub fn to_cells(&self) -> Vec<Vec<Option<Color>>> {
        let mut result = vec![];
        for row in 0..self.size() {
            let mut cells_in_row = vec![];
            for column in 0..self.size() {
                cells_in_row.push(self.get_color(Coords { row, column }));
            }
            result.push(cells_in_row);
        }
        result
    }

    pub fn size(&self) -> CoordValue {
        self.cells.size
    }

    pub fn get_color(&self, coords: Coords) -> Option<Color> {
        self.cells.at_coord(coords).color
    }

    fn get_color_of_position(&self, position: Position) -> Option<Color> {
        match position {
            Position::Top | Position::Bottom => Some(Color::Black),
            Position::Left | Position::Right => Some(Color::White),
            Position::Index(index) => self.cells.at_index(index).color,
        }
    }

    fn get_neighbors(&self, index: Index) -> impl Iterator<Item = Position> {
        let size = self.cells.size as Index;
        let left_neighbor = if index % size == 0 {
            Position::Left
        } else {
            Position::Index(index - 1)
        };

        let top_left_neighbor = if index < size {
            Position::Top
        } else {
            Position::Index(index - size)
        };

        let top_right_neighbor = if index >= size && index % size < size - 1 {
            Some(Position::Index(index - size + 1))
        } else {
            None
        };

        let right_neighbor = if index % size == size - 1 {
            Position::Right
        } else {
            Position::Index(index + 1)
        };

        let bottom_right_neighbor = if index >= size * (size - 1) {
            Position::Bottom
        } else {
            Position::Index(index + size)
        };

        let bottom_left_neighbor = if index < size * (size - 1) && index % size > 0 {
            Some(Position::Index(index + size - 1))
        } else {
            None
        };

        iter::empty()
            .chain(iter::once(left_neighbor))
            .chain(iter::once(top_left_neighbor))
            .chain(top_right_neighbor)
            .chain(iter::once(right_neighbor))
            .chain(iter::once(bottom_right_neighbor))
            .chain(bottom_left_neighbor)
    }

    pub fn play(&mut self, coords: Coords, color: Color) -> Result<(), InvalidMove> {
        if coords.row >= self.size() || coords.column >= self.size() {
            return Err(InvalidMove::OutOfBounds(coords));
        }

        let index = self.cells.index_from_coords(coords);
        let position = Position::Index(index);

        if self.cells.at_index(index).color.is_some() {
            return Err(InvalidMove::CellOccupied(coords));
        }

        self.cells.set_index(
            index,
            Cell {
                color: Some(color),
                parent: None,
            },
        );

        self.merge_with_neighbors(index, position, color);
        Ok(())
    }

    fn merge_with_neighbors(&mut self, index: Index, position: Position, color: Color) {
        let mut iter = self.get_neighbors(index);

        while let Some(neighbor) = iter.next() {
            if self.get_color_of_position(neighbor) == Some(color) {
                self.merge(position, neighbor);
                // After merging with one neighbor, we can skip the next one:
                // If the next neighbor also has the same color,
                // then it must already be part of the same set.
                iter.next();
            }
        }
    }

    pub fn get_empty_cells(&self) -> Vec<Coords> {
        let mut result = vec![];
        for row in 0..self.size() {
            for column in 0..self.size() {
                let coords = Coords { row, column };
                if self.get_color(coords).is_none() {
                    result.push(coords);
                }
            }
        }

        result
    }
}

impl UnionFind<Position> for Board {
    fn get_parent(&self, item: Position) -> Option<Position> {
        match item {
            Position::Top => self.top_parent,
            Position::Left => self.left_parent,
            Position::Bottom | Position::Right => None,
            Position::Index(index) => self.cells.at_index(index).parent,
        }
    }

    fn set_parent(&mut self, item: Position, parent: Position) {
        match item {
            Position::Top => {
                self.top_parent = Some(parent);
            }
            Position::Left => {
                self.left_parent = Some(parent);
            }
            Position::Bottom | Position::Right => panic!("Cannot set parent of {:?}", item),
            Position::Index(index) => {
                let cell = self.cells.at_index(index);
                self.cells.set_index(
                    index,
                    Cell {
                        parent: Some(parent),
                        ..cell
                    },
                );
            }
        }
    }
}

fn check_board_size(input: usize) -> Result<CoordValue, InvalidBoard> {
    input
        .try_into()
        .ok()
        .filter(|&size| MIN_BOARD_SIZE <= size && size <= MAX_BOARD_SIZE)
        .ok_or(InvalidBoard::SizeOutOfBounds(
            input,
            MIN_BOARD_SIZE,
            MAX_BOARD_SIZE,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let board = Board::new(3);
        assert_eq!(board.size(), 3);
        assert!(board.get_color(Coords { row: 0, column: 0 }).is_none());
    }

    #[test]
    fn test_from_cells() {
        let cells = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None],
        ];
        let board = Board::from_cells(cells).unwrap();
        assert_eq!(board.get_color(Coords { row: 0, column: 0 }), None);
        assert_eq!(
            board.get_color(Coords { row: 0, column: 1 }),
            Some(Color::Black)
        );
        assert_eq!(
            board.get_color(Coords { row: 1, column: 0 }),
            Some(Color::White)
        );
        assert_eq!(board.get_color(Coords { row: 1, column: 1 }), None);
    }

    #[test]
    fn test_from_cells_with_size_too_small() {
        let cells = vec![vec![Some(Color::Black)]];
        let error = Board::from_cells(cells).err().unwrap();
        assert_eq!(
            error,
            InvalidBoard::SizeOutOfBounds(1, MIN_BOARD_SIZE, MAX_BOARD_SIZE)
        );
    }

    #[test]
    fn test_from_cells_with_size_too_large() {
        let invalid_size = MAX_BOARD_SIZE as usize + 1;
        let row: Vec<Option<Color>> = vec![None; invalid_size];
        let mut cells = vec![];
        for _ in 0..invalid_size {
            cells.push(row.clone());
        }
        let error = Board::from_cells(cells).err().unwrap();
        assert_eq!(
            error,
            InvalidBoard::SizeOutOfBounds(invalid_size, MIN_BOARD_SIZE, MAX_BOARD_SIZE)
        );
    }

    #[test]
    fn test_from_cells_with_non_square_board() {
        let cells = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None, None],
        ];
        let error = Board::from_cells(cells).err().unwrap();
        assert_eq!(error, InvalidBoard::NotSquare(1, 2));
    }

    #[test]
    fn test_to_cells() {
        let mut board = Board::new(2);
        board
            .play(Coords { row: 0, column: 1 }, Color::Black)
            .unwrap();
        board
            .play(Coords { row: 1, column: 0 }, Color::White)
            .unwrap();
        let expected_cells = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None],
        ];
        assert_eq!(board.to_cells(), expected_cells);
    }

    #[test]
    fn test_play() {
        let mut board = Board::new(3);
        let coords = Coords { row: 1, column: 2 };
        let result = board.play(coords, Color::Black);
        assert!(result.is_ok());
        assert_eq!(board.get_color(coords).unwrap(), Color::Black);
    }

    #[test]
    fn test_play_row_out_of_bounds() {
        let mut board = Board::new(3);
        let coords = Coords { row: 3, column: 2 };
        let error = board.play(coords, Color::Black).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds(coords));
    }

    #[test]
    fn test_play_column_out_of_bounds() {
        let mut board = Board::new(3);
        let coords = Coords { row: 0, column: 3 };
        let error = board.play(coords, Color::Black).unwrap_err();
        assert_eq!(error, InvalidMove::OutOfBounds(coords));
    }

    #[test]
    fn test_play_on_occupied_cell() {
        let mut board = Board::new(3);
        let coords = Coords { row: 1, column: 2 };
        board.play(coords, Color::Black).unwrap();
        let error = board.play(coords, Color::Black).unwrap_err();
        assert_eq!(error, InvalidMove::CellOccupied(coords));
    }

    #[test]
    fn test_merge_neighbors_of_own_color() {
        let mut board = Board::new(5);
        let center = Coords { row: 2, column: 2 };
        let center_index = Position::Index(board.cells.index_from_coords(center));
        let neighbor1 = Coords { row: 1, column: 2 };
        let neighbor1_index = Position::Index(board.cells.index_from_coords(neighbor1));
        let neighbor2 = Coords { row: 3, column: 2 };
        let neighbor2_index = Position::Index(board.cells.index_from_coords(neighbor2));
        board.play(neighbor1, Color::Black).unwrap();
        board.play(neighbor2, Color::Black).unwrap();

        assert!(!board.is_in_same_set(neighbor1_index, neighbor2_index));

        board.play(center, Color::Black).unwrap();

        assert!(board.is_in_same_set(neighbor1_index, neighbor2_index));
        assert!(board.is_in_same_set(center_index, neighbor2_index));
    }

    #[test]
    fn test_do_not_merge_neighbors_of_other_color() {
        let mut board = Board::new(5);
        let center = Coords { row: 2, column: 2 };
        let center_index = Position::Index(board.cells.index_from_coords(center));
        let neighbor = Coords { row: 1, column: 2 };
        let neighbor_index = Position::Index(board.cells.index_from_coords(neighbor));

        board.play(neighbor, Color::White).unwrap();
        board.play(center, Color::Black).unwrap();

        assert!(!board.is_in_same_set(center_index, neighbor_index));
    }

    #[test]
    fn test_do_not_merge_cells_that_are_not_connected() {
        let mut board = Board::new(3);
        let top_left = Coords { row: 0, column: 0 };
        let top_left_index = Position::Index(board.cells.index_from_coords(top_left));
        let bottom_right = Coords { row: 2, column: 2 };
        let bottom_right_index = Position::Index(board.cells.index_from_coords(bottom_right));

        board.play(top_left, Color::Black).unwrap();
        board.play(bottom_right, Color::Black).unwrap();

        assert!(!board.is_in_same_set(top_left_index, bottom_right_index));
    }

    #[test]
    fn test_get_empty_cells() {
        let mut board = Board::new(2);
        board
            .play(Coords { row: 0, column: 0 }, Color::Black)
            .unwrap();
        board
            .play(Coords { row: 1, column: 1 }, Color::White)
            .unwrap();

        assert_eq!(
            board.get_empty_cells(),
            vec![Coords { row: 0, column: 1 }, Coords { row: 1, column: 0 },]
        );
    }
}

#[cfg(test)]
mod test_neighbors {
    use super::*;

    fn check_neighbors(board: &Board, row: CoordValue, column: CoordValue, expected: &[Position]) {
        let index = board.cells.index_from_coords(Coords { row, column });
        let neighbors: Vec<Position> = board.get_neighbors(index).collect();
        assert_eq!(neighbors, expected);
    }

    fn make_position(board: &Board, row: CoordValue, column: CoordValue) -> Position {
        Position::Index(board.cells.index_from_coords(Coords { row, column }))
    }

    #[test]
    fn test_neighbors_top_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 0, &[
            Position::Left,
            Position::Top,
            make_position(&board, 0, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 1, &[
            make_position(&board, 0, 0),
            Position::Top,
            make_position(&board, 0, 2),
            make_position(&board, 1, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 0, &[
            Position::Left,
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
            make_position(&board, 0, 3),
            Position::Top,
            Position::Right,
            make_position(&board, 1, 4),
            make_position(&board, 1, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 3, &[
            make_position(&board, 0, 2),
            Position::Top,
            make_position(&board, 0, 4),
            make_position(&board, 1, 3),
            make_position(&board, 1, 2),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 4, &[
            make_position(&board, 1, 3),
            make_position(&board, 0, 4),
            Position::Right,
            make_position(&board, 2, 4),
            make_position(&board, 2, 3),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 0, &[
            Position::Left,
            make_position(&board, 3, 0),
            make_position(&board, 3, 1),
            make_position(&board, 4, 1),
            Position::Bottom,
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 0, &[
            Position::Left,
            make_position(&board, 2, 0),
            make_position(&board, 2, 1),
            make_position(&board, 3, 1),
            make_position(&board, 4, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 1, &[
            make_position(&board, 4, 0),
            make_position(&board, 3, 1),
            make_position(&board, 3, 2),
            make_position(&board, 4, 2),
            Position::Bottom,
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 4, &[
            make_position(&board, 4, 3),
            make_position(&board, 3, 4),
            Position::Right,
            Position::Bottom,
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 4, &[
            make_position(&board, 3, 3),
            make_position(&board, 2, 4),
            Position::Right,
            make_position(&board, 4, 4),
            make_position(&board, 4, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 3, &[
            make_position(&board, 4, 2),
            make_position(&board, 3, 3),
            make_position(&board, 3, 4),
            make_position(&board, 4, 4),
            Position::Bottom,
        ]);
    }

    #[test]
    fn test_neighbors_center() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 2, 2, &[
            make_position(&board, 2, 1),
            make_position(&board, 1, 2),
            make_position(&board, 1, 3),
            make_position(&board, 2, 3),
            make_position(&board, 3, 2),
            make_position(&board, 3, 1),
        ]);
    }
}
