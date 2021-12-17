use crate::coords::Coords;
use crate::errors::{InvalidBoard, InvalidMove};
use crate::square_array::SquareArray;
use crate::union_find::UnionFind;
use std::convert::TryInto;

// Neighbor calculations assume size >= 2
pub const MIN_BOARD_SIZE: u8 = 2;
// Technically, we support much larger boards, but future optimizations may restrict this.
pub const MAX_BOARD_SIZE: u8 = 19;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Color {
    BLACK,
    WHITE,
}

impl Color {
    pub fn opponent_color(&self) -> Self {
        match self {
            Color::BLACK => Color::WHITE,
            Color::WHITE => Color::BLACK,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Position {
    // Order is important here: UnionFind's merge always chooses larger positions as roots.
    // Thus, BOTTOM and RIGHT are always their own parent and we do not have to store their parents.
    Index(u16),
    TOP,
    LEFT,
    BOTTOM,
    RIGHT,
}

#[derive(Copy, Clone, Default)]
struct Cell {
    color: Option<Color>,
    parent: Option<Position>,
}

#[derive(Clone)]
pub struct Board {
    cells: SquareArray<Cell>,
    top_parent: Position,
    left_parent: Position,
}

impl Board {
    pub fn new(size: u8) -> Self {
        check_board_size(size).expect("Invalid size");
        Self {
            cells: SquareArray::new(size),
            top_parent: Position::TOP,
            left_parent: Position::LEFT,
        }
    }

    pub fn from_cells(cells: Vec<Vec<Option<Color>>>) -> Result<Self, InvalidBoard> {
        let size = check_board_size(cells.len())?;
        let mut board = Self::new(size);

        for (row, cells_in_row) in (0..size).zip(cells) {
            if cells_in_row.len() != size as usize {
                return Err(InvalidBoard::NotSquare(row as u8, size));
            }
            for (column, cell) in (0..size).zip(cells_in_row) {
                if let Some(color) = cell {
                    board.play(Coords { row, column }, color).unwrap();
                }
            }
        }
        Ok(board)
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
        // This method guarantees that neighbors are returned in clock-wise order, starting with the left neighbor.
        let mut neighbors = Vec::new();
        let size: u16 = self.size().into();

        if index % size == 0 {
            neighbors.push(Position::LEFT);
        } else {
            neighbors.push(Position::Index(index - 1));
        }

        if index < size {
            neighbors.push(Position::TOP);
        } else {
            neighbors.push(Position::Index(index - size));
            if index % size < size - 1 {
                neighbors.push(Position::Index(index - size + 1));
            }
        }

        if index % size == size - 1 {
            neighbors.push(Position::RIGHT);
        } else {
            neighbors.push(Position::Index(index + 1));
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
                parent: Some(position),
            },
        );

        for neighbor in self.get_neighbors(index) {
            if self.get_color_of_position(neighbor) == Some(color) {
                self.merge(position, neighbor);
            }
        }

        Ok(())
    }

    pub fn get_empty_cells(&self) -> Vec<Coords> {
        let mut result = vec![];
        for row in 0..self.size() {
            for column in 0..self.size() {
                let coords = Coords { row, column };
                if let None = self.get_color(coords) {
                    result.push(coords);
                }
            }
        }

        result
    }
}

impl UnionFind<Position> for Board {
    fn get_parent(&self, item: Position) -> Position {
        match item {
            Position::TOP => self.top_parent,
            Position::LEFT => self.left_parent,
            Position::BOTTOM | Position::RIGHT => item,
            Position::Index(index) => self.cells.at_index(index).parent.unwrap_or(item),
        }
    }

    fn set_parent(&mut self, item: Position, parent: Position) {
        match item {
            Position::TOP => {
                self.top_parent = parent;
            }
            Position::LEFT => {
                self.left_parent = parent;
            }
            Position::BOTTOM | Position::RIGHT => panic!("Cannot set parent of {:?}", item),
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

fn check_board_size<T: Copy + TryInto<u8> + Into<usize>>(input: T) -> Result<u8, InvalidBoard> {
    input
        .try_into()
        .ok()
        .filter(|&size| MIN_BOARD_SIZE <= size && size <= MAX_BOARD_SIZE)
        .ok_or(InvalidBoard::SizeOutOfBounds(
            input.into(),
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
            vec![None, Some(Color::BLACK)],
            vec![Some(Color::WHITE), None],
        ];
        let board = Board::from_cells(cells).unwrap();
        assert_eq!(board.get_color(Coords { row: 0, column: 0 }), None);
        assert_eq!(
            board.get_color(Coords { row: 0, column: 1 }),
            Some(Color::BLACK)
        );
        assert_eq!(
            board.get_color(Coords { row: 1, column: 0 }),
            Some(Color::WHITE)
        );
        assert_eq!(board.get_color(Coords { row: 1, column: 1 }), None);
    }

    #[test]
    fn test_from_cells_with_size_too_small() {
        let cells = vec![vec![Some(Color::BLACK)]];
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
            vec![None, Some(Color::BLACK)],
            vec![Some(Color::WHITE), None, None],
        ];
        let error = Board::from_cells(cells).err().unwrap();
        assert_eq!(error, InvalidBoard::NotSquare(1, 2));
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

    #[test]
    fn test_merge_neighbors_of_own_color() {
        let mut board = Board::new(5);
        let center = Coords { row: 2, column: 2 };
        let center_index = Position::Index(board.cells.index_from_coords(center));
        let neighbor1 = Coords { row: 1, column: 2 };
        let neighbor1_index = Position::Index(board.cells.index_from_coords(neighbor1));
        let neighbor2 = Coords { row: 3, column: 2 };
        let neighbor2_index = Position::Index(board.cells.index_from_coords(neighbor2));
        let _ = board.play(neighbor1, Color::BLACK);
        let _ = board.play(neighbor2, Color::BLACK);

        assert!(!board.is_in_same_set(neighbor1_index, neighbor2_index));

        let _ = board.play(center, Color::BLACK);

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

        let _ = board.play(neighbor, Color::WHITE);
        let _ = board.play(center, Color::BLACK);

        assert!(!board.is_in_same_set(center_index, neighbor_index));
    }

    #[test]
    fn test_do_not_merge_cells_that_are_not_connected() {
        let mut board = Board::new(3);
        let top_left = Coords { row: 0, column: 0 };
        let top_left_index = Position::Index(board.cells.index_from_coords(top_left));
        let bottom_right = Coords { row: 2, column: 2 };
        let bottom_right_index = Position::Index(board.cells.index_from_coords(bottom_right));

        let _ = board.play(top_left, Color::BLACK);
        let _ = board.play(bottom_right, Color::BLACK);

        assert!(!board.is_in_same_set(top_left_index, bottom_right_index));
    }

    #[test]
    fn test_get_empty_cells() {
        let mut board = Board::new(2);
        let _ = board.play(Coords { row: 0, column: 0 }, Color::BLACK);
        let _ = board.play(Coords { row: 1, column: 1 }, Color::WHITE);

        assert_eq!(
            board.get_empty_cells(),
            vec![Coords { row: 0, column: 1 }, Coords { row: 1, column: 0 },]
        );
    }
}

#[cfg(test)]
mod test_neighbors {
    use super::*;

    fn check_neighbors(board: &Board, row: u8, column: u8, expected: &[Position]) {
        let index = board.cells.index_from_coords(Coords { row, column });
        let neighbors = board.get_neighbors(index);
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
            Position::LEFT,
            Position::TOP,
            make_position(&board, 0, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 1, &[
            make_position(&board, 0, 0),
            Position::TOP,
            make_position(&board, 0, 2),
            make_position(&board, 1, 1),
            make_position(&board, 1, 0),
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
            make_position(&board, 0, 3),
            Position::TOP,
            Position::RIGHT,
            make_position(&board, 1, 4),
            make_position(&board, 1, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 3, &[
            make_position(&board, 0, 2),
            Position::TOP,
            make_position(&board, 0, 4),
            make_position(&board, 1, 3),
            make_position(&board, 1, 2),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 4, &[
            make_position(&board, 1, 3),
            make_position(&board, 0, 4),
            Position::RIGHT,
            make_position(&board, 2, 4),
            make_position(&board, 2, 3),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 0, &[
            Position::LEFT,
            make_position(&board, 3, 0),
            make_position(&board, 3, 1),
            make_position(&board, 4, 1),
            Position::BOTTOM,
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
            make_position(&board, 4, 0),
            make_position(&board, 3, 1),
            make_position(&board, 3, 2),
            make_position(&board, 4, 2),
            Position::BOTTOM,
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 4, &[
            make_position(&board, 4, 3),
            make_position(&board, 3, 4),
            Position::RIGHT,
            Position::BOTTOM,
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 4, &[
            make_position(&board, 3, 3),
            make_position(&board, 2, 4),
            Position::RIGHT,
            make_position(&board, 4, 4),
            make_position(&board, 4, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 3, &[
            make_position(&board, 4, 2),
            make_position(&board, 3, 3),
            make_position(&board, 3, 4),
            make_position(&board, 4, 4),
            Position::BOTTOM,
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
