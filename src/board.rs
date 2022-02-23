use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::errors::{InvalidBoard, InvalidMove};
use crate::hex_cells::{HexCells, Index};
use crate::union_find::UnionFind;
use std::iter;
use std::iter::Iterator;

/// Minimal supported board size
/// # (Some neighbor calculations assume that the size is at least 2.)
pub const MIN_BOARD_SIZE: CoordValue = 2;
/// Maximal supported board size
/// # (Technically, we support much larger boards, but future optimizations may restrict this.)
pub const MAX_BOARD_SIZE: CoordValue = 19;

/// This type represents the board as a matrix of `Option<Color>`. It may be used to serialize/deserialize boards.
///
/// Each entry represents the stone placed on the respective cell, with `None` representing an empty cell.
///
/// The following board will be represented as `vec![vec![None, Some(Color::Black)], vec![Some(Color::White), None]]`:
/// ```text
///  0  1
/// 0\.  ●\0
///  1\○  .\1
///     0  1
/// ```
///
pub type StoneMatrix = Vec<Vec<Option<Color>>>;

/// `Board` represents the Hex board and all placed stones.
///
/// The `play` method can be used to place stones on the board.
/// Note that `Board` has no notion of a current player and will allow to place any amount of stones in any color.
///
/// A nice human-readable format can be obtained via the `Display` trait:
/// ```
/// use hexgame::{Board, Color, Coords}
/// let mut board = Board::new(5);
/// board.play(Coords::new(1, 3), Color::Black);
/// board.play(Coords::new(0, 2), Color::White);
/// println!("{}", board);
/// ```
/// will output
/// ```text
///  a  b  c  d  e
/// 1\.  .  ○  .  .\1
///  2\.  .  .  ●  .\2
///   3\.  .  .  .  .\3
///    4\.  .  .  .  .\4
///     5\.  .  .  .  .\5
///        a  b  c  d  e
/// ```
#[derive(Clone)]
pub struct Board {
    cells: HexCells,
}

impl Board {
    /// Create a new board with the given size. Boards are always square.
    ///
    /// This method will panic if the size is not bounded by `MIN_BOARD_SIZE` and `MAX_BOARD_SIZE`.
    pub fn new(size: CoordValue) -> Self {
        check_board_size(size as usize).expect("Invalid size");
        let mut board = Self {
            cells: HexCells::new(size),
        };
        for color in [Color::Black, Color::White] {
            for edge in board.get_edges(color) {
                board.cells.set_color_at_index(edge, color);
            }
        }
        board
    }

    /// Load a board from a `StoneMatrix`.
    pub fn from_stone_matrix(stones: StoneMatrix) -> Result<Self, InvalidBoard> {
        let size = check_board_size(stones.len())?;
        let mut board = Self::new(size);

        for (row, stones_in_row) in stones.into_iter().enumerate() {
            if stones_in_row.len() != size as usize {
                return Err(InvalidBoard::NotSquare(row as u8, size));
            }
            for (column, cell) in stones_in_row.into_iter().enumerate() {
                if let Some(color) = cell {
                    let row = row as u8;
                    let column = column as u8;
                    board.play(Coords { row, column }, color).unwrap();
                }
            }
        }
        Ok(board)
    }

    /// Convert this board to a `StoneMatrix`.
    pub fn to_stone_matrix(&self) -> StoneMatrix {
        (0..self.size())
            .map(|row| {
                (0..self.size())
                    .map(|column| self.get_color(Coords::new(row, column)))
                    .collect()
            })
            .collect()
    }

    /// Return the size of this board. Boards are always square.
    pub fn size(&self) -> CoordValue {
        self.cells.size
    }

    /// Return the color at the given coordinates.
    /// If no stone has been placed in the given cell, this method will return None.
    pub fn get_color(&self, coords: Coords) -> Option<Color> {
        self.cells.get_color_at_coords(coords)
    }

    fn get_color_at_index(&self, index: Index) -> Option<Color> {
        self.cells.get_color_at_index(index)
    }

    /// TODO: can we restrict the visibility this method and is_in_same_set to this crate?
    /// These are the only methods that expose `Index`.
    pub fn get_edges(&self, color: Color) -> [Index; 2] {
        match color {
            Color::Black => [self.cells.top(), self.cells.bottom()],
            Color::White => [self.cells.left(), self.cells.right()],
        }
    }

    pub fn is_in_same_set(&self, index1: Index, index2: Index) -> bool {
        self.cells.is_in_same_set(index1, index2)
    }

    fn get_neighbors(&self, index: Index) -> impl Iterator<Item = Index> {
        let size = self.cells.size as Index;
        let left_neighbor = if index % size == 0 {
            self.cells.left()
        } else {
            index - 1
        };

        let top_left_neighbor = if index < size {
            self.cells.top()
        } else {
            index - size
        };

        let top_right_neighbor = if index >= size && index % size < size - 1 {
            Some(index - size + 1)
        } else {
            None
        };

        let right_neighbor = if index % size == size - 1 {
            self.cells.right()
        } else {
            index + 1
        };

        let bottom_right_neighbor = if index >= size * (size - 1) {
            self.cells.bottom()
        } else {
            index + size
        };

        let bottom_left_neighbor = if index < size * (size - 1) && index % size > 0 {
            Some(index + size - 1)
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

        if self.get_color_at_index(index).is_some() {
            return Err(InvalidMove::CellOccupied(coords));
        }

        self.cells.set_color_at_index(index, color);
        self.merge_with_neighbors(index, color);

        Ok(())
    }

    fn merge_with_neighbors(&mut self, index: Index, color: Color) {
        let mut iter = self.get_neighbors(index);

        while let Some(neighbor) = iter.next() {
            if self.get_color_at_index(neighbor) == Some(color) {
                self.cells.merge(index, neighbor);
                // After merging with one neighbor, we can skip the next one:
                // If the next neighbor also has the same color,
                // then it must already be part of the same set.
                iter.next();
            }
        }
    }

    /// Return all empty cells.
    pub fn get_empty_cells(&self) -> Vec<Coords> {
        let size = self.size() as usize;
        let mut result = Vec::with_capacity(size * size);
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

    pub fn find_attacked_bridges(&self, coords: Coords) -> Vec<Coords> {
        let center_index = self.cells.index_from_coords(coords);
        let search_color = match self.get_color_at_index(center_index) {
            None => return vec![],
            Some(color) => color.opponent_color(),
        };

        // Essentially, the following code will search for the pattern [Some(search_color), None, Some(search_color)]
        // in the colors of the neighbors of `coords`. To find bridges that span the

        // Bridges may cross the end of the get_neighbors-iterator (e.g. the last element + the first two elements).
        // For this reason we extend the array of neighbors by the first two neighbors.
        // Unfortunately, we cannot know the number of neighbors (typically 6, but less on the edges).
        // Note that we initialize the array with `center_index` which is guaranteed to not match any part of the pattern.
        let mut neighbors: [Index; 8] = [center_index; 8];
        let neighbor_count = copy_into_array(&mut neighbors, 0, self.get_neighbors(center_index));
        neighbors[neighbor_count] = neighbors[0];
        neighbors[neighbor_count + 1] = neighbors[1];

        let mut result = vec![];
        // `state` describes how many fields of the pattern [Some(search_color), None, Some(search_color)] we've already found.
        let mut state = FindAttackedBridgesState::Found0;

        for (i, neighbor) in neighbors.iter().enumerate() {
            let color = self.get_color_at_index(*neighbor);
            match state {
                FindAttackedBridgesState::Found0 => {
                    if color == Some(search_color) {
                        state = FindAttackedBridgesState::Found1;
                    }
                }
                FindAttackedBridgesState::Found1 => {
                    if color.is_none() {
                        state = FindAttackedBridgesState::Found2;
                    } else if color != Some(search_color) {
                        state = FindAttackedBridgesState::Found0;
                    }
                    // else: remain in state Found1, because we already have the beginning of a possible new match.
                }
                FindAttackedBridgesState::Found2 => {
                    if color == Some(search_color) {
                        result.push(self.cells.coords_from_index(neighbors[i - 1]));
                        // It is important to also find overlapping matches.
                        // The third part of the current bridge may be the first part of a new bridge.
                        state = FindAttackedBridgesState::Found1;
                    } else {
                        state = FindAttackedBridgesState::Found0;
                    }
                }
            }
        }

        result
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

fn copy_into_array<T, TIter: Iterator<Item = T>, const N: usize>(
    array: &mut [T; N],
    start_index: usize,
    iterator: TIter,
) -> usize {
    let mut index = start_index;
    for item in iterator {
        array[index] = item;
        index += 1;
    }
    index
}

enum FindAttackedBridgesState {
    Found0,
    Found1,
    Found2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let board = Board::new(3);
        assert_eq!(board.size(), 3);
        assert!(board.get_color(Coords { row: 0, column: 0 }).is_none());
        assert_eq!(
            board.get_color_at_index(board.cells.left()),
            Some(Color::White)
        );
        assert_eq!(
            board.get_color_at_index(board.cells.top()),
            Some(Color::Black)
        );
        assert_eq!(
            board.get_color_at_index(board.cells.bottom()),
            Some(Color::Black)
        );
        assert_eq!(
            board.get_color_at_index(board.cells.left()),
            Some(Color::White)
        );
    }

    #[test]
    fn test_from_stone_matrix() {
        let cells = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None],
        ];
        let board = Board::from_stone_matrix(cells).unwrap();
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
    fn test_from_stone_matrix_with_size_too_small() {
        let cells = vec![vec![Some(Color::Black)]];
        let error = Board::from_stone_matrix(cells).err().unwrap();
        assert_eq!(
            error,
            InvalidBoard::SizeOutOfBounds(1, MIN_BOARD_SIZE, MAX_BOARD_SIZE)
        );
    }

    #[test]
    fn test_from_stone_matrix_with_size_too_large() {
        let invalid_size = MAX_BOARD_SIZE as usize + 1;
        let row: Vec<Option<Color>> = vec![None; invalid_size];
        let mut cells = vec![];
        for _ in 0..invalid_size {
            cells.push(row.clone());
        }
        let error = Board::from_stone_matrix(cells).err().unwrap();
        assert_eq!(
            error,
            InvalidBoard::SizeOutOfBounds(invalid_size, MIN_BOARD_SIZE, MAX_BOARD_SIZE)
        );
    }

    #[test]
    fn test_from_stone_matrix_with_non_square_board() {
        let cells = vec![
            vec![None, Some(Color::Black)],
            vec![Some(Color::White), None, None],
        ];
        let error = Board::from_stone_matrix(cells).err().unwrap();
        assert_eq!(error, InvalidBoard::NotSquare(1, 2));
    }

    #[test]
    fn test_to_stone_matrix() {
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
        assert_eq!(board.to_stone_matrix(), expected_cells);
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
        let center_index = board.cells.index_from_coords(center);
        let neighbor1 = Coords { row: 1, column: 2 };
        let neighbor1_index = board.cells.index_from_coords(neighbor1);
        let neighbor2 = Coords { row: 3, column: 2 };
        let neighbor2_index = board.cells.index_from_coords(neighbor2);
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
        let center_index = board.cells.index_from_coords(center);
        let neighbor = Coords { row: 1, column: 2 };
        let neighbor_index = board.cells.index_from_coords(neighbor);

        board.play(neighbor, Color::White).unwrap();
        board.play(center, Color::Black).unwrap();

        assert!(!board.is_in_same_set(center_index, neighbor_index));
    }

    #[test]
    fn test_do_not_merge_cells_that_are_not_connected() {
        let mut board = Board::new(3);
        let top_left = Coords { row: 0, column: 0 };
        let top_left_index = board.cells.index_from_coords(top_left);
        let bottom_right = Coords { row: 2, column: 2 };
        let bottom_right_index = board.cells.index_from_coords(bottom_right);

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

    fn check_neighbors(board: &Board, row: CoordValue, column: CoordValue, expected: &[Index]) {
        let index = board.cells.index_from_coords(Coords { row, column });
        let neighbors: Vec<Index> = board.get_neighbors(index).collect();
        assert_eq!(neighbors, expected);
    }

    fn make_position(board: &Board, row: CoordValue, column: CoordValue) -> Index {
        board.cells.index_from_coords(Coords::new(row, column))
    }

    #[test]
    fn test_neighbors_top_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 0, &[
            board.cells.left(),
            board.cells.top(),
            make_position(&board, 0, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 1, &[
            make_position(&board, 0, 0),
            board.cells.top(),
            make_position(&board, 0, 2),
            make_position(&board, 1, 1),
            make_position(&board, 1, 0),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 0, &[
            board.cells.left(),
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
            board.cells.top(),
            board.cells.right(),
            make_position(&board, 1, 4),
            make_position(&board, 1, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 0, 3, &[
            make_position(&board, 0, 2),
            board.cells.top(),
            make_position(&board, 0, 4),
            make_position(&board, 1, 3),
            make_position(&board, 1, 2),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 1, 4, &[
            make_position(&board, 1, 3),
            make_position(&board, 0, 4),
            board.cells.right(),
            make_position(&board, 2, 4),
            make_position(&board, 2, 3),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 0, &[
            board.cells.left(),
            make_position(&board, 3, 0),
            make_position(&board, 3, 1),
            make_position(&board, 4, 1),
            board.cells.bottom(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 0, &[
            board.cells.left(),
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
            board.cells.bottom(),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let board = Board::new(5);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 4, &[
            make_position(&board, 4, 3),
            make_position(&board, 3, 4),
            board.cells.right(),
            board.cells.bottom(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 3, 4, &[
            make_position(&board, 3, 3),
            make_position(&board, 2, 4),
            board.cells.right(),
            make_position(&board, 4, 4),
            make_position(&board, 4, 3),
        ]);
        #[rustfmt::skip]
        check_neighbors(&board, 4, 3, &[
            make_position(&board, 4, 2),
            make_position(&board, 3, 3),
            make_position(&board, 3, 4),
            make_position(&board, 4, 4),
            board.cells.bottom(),
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

    #[cfg(test)]
    mod test_attacked_bridges {
        use super::*;
        const CENTER: Coords = Coords { row: 2, column: 2 };

        #[test]
        fn test_coords_without_stone() {
            let board = Board::new(5);
            assert_eq!(board.find_attacked_bridges(CENTER), vec![]);
        }

        #[test]
        fn test_coords_without_neighbors() {
            let mut board = Board::new(5);
            let _ = board.play(CENTER, Color::White);
            assert_eq!(board.find_attacked_bridges(CENTER), vec![]);
        }

        #[test]
        fn test_simple_bridge_in_center() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  .  ●  .\2
            //   3\.  .  ○  .  .\3
            //    4\.  .  ●  .  .\4
            let mut board = Board::new(5);
            let _ = board.play(Coords { row: 1, column: 3 }, Color::Black);
            let _ = board.play(Coords { row: 3, column: 2 }, Color::Black);
            let _ = board.play(CENTER, Color::White);

            assert_eq!(
                board.find_attacked_bridges(CENTER),
                vec![Coords { row: 2, column: 3 }]
            );
        }

        #[test]
        fn test_bridge_with_preceding_stone_in_center() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  ●  .  .\2
            //   3\.  ●  ○  ●  .\3
            let mut board = Board::new(5);
            let _ = board.play(Coords { row: 2, column: 1 }, Color::Black);
            let _ = board.play(Coords { row: 1, column: 2 }, Color::Black);
            let _ = board.play(Coords { row: 2, column: 3 }, Color::Black);
            let _ = board.play(CENTER, Color::White);

            assert_eq!(
                board.find_attacked_bridges(CENTER),
                vec![Coords { row: 1, column: 3 }]
            );
        }

        #[test]
        fn test_non_bridge_with_preceding_stone_in_center() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  ○  .  .\2
            //   3\.  ●  ○  ●  .\3
            let mut board = Board::new(5);
            let _ = board.play(Coords { row: 2, column: 1 }, Color::Black);
            let _ = board.play(Coords { row: 1, column: 2 }, Color::White);
            let _ = board.play(Coords { row: 2, column: 3 }, Color::Black);
            let _ = board.play(CENTER, Color::White);

            assert_eq!(board.find_attacked_bridges(CENTER), vec![]);
        }

        #[test]
        fn test_three_bridges_in_center() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  .  ●  .\2
            //   3\.  ●  ○  .  .\3
            //    4\.  .  ●  .  .\4
            let mut board = Board::new(5);
            let _ = board.play(Coords { row: 2, column: 1 }, Color::Black);
            let _ = board.play(Coords { row: 1, column: 3 }, Color::Black);
            let _ = board.play(Coords { row: 3, column: 2 }, Color::Black);
            let _ = board.play(CENTER, Color::White);

            assert_eq!(
                board.find_attacked_bridges(CENTER),
                vec![
                    Coords { row: 1, column: 2 },
                    Coords { row: 2, column: 3 },
                    Coords { row: 3, column: 1 },
                ]
            );
        }

        #[test]
        fn test_bridge_overlapping_from_last_to_first_neighbor() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  ●  .  .\2
            //   3\.  .  ○  .  .\3
            //    4\.  ●  .  .  .\4
            let mut board = Board::new(5);
            let _ = board.play(Coords { row: 3, column: 1 }, Color::Black);
            let _ = board.play(Coords { row: 1, column: 2 }, Color::Black);
            let _ = board.play(CENTER, Color::White);

            assert_eq!(
                board.find_attacked_bridges(CENTER),
                vec![Coords { row: 2, column: 1 },]
            );
        }

        #[test]
        fn test_bridge_in_obtuse_corner() {
            //  a  b  c  d  e
            // 1\.  ○  .  .  .\1
            //  2\●  .  .  .  .\2
            let mut board = Board::new(5);
            let attacked_coords = Coords { row: 0, column: 1 };
            let _ = board.play(Coords { row: 1, column: 0 }, Color::Black);
            let _ = board.play(attacked_coords, Color::White);

            assert_eq!(
                board.find_attacked_bridges(attacked_coords),
                vec![Coords { row: 0, column: 0 },]
            );
        }

        #[test]
        fn test_bridge_next_to_obtuse_corner() {
            //  a  b  c  d  e
            // 1\○  .  .  .  .\1
            //  2\●  .  .  .  .\2
            let mut board = Board::new(5);
            let attacked_coords = Coords { row: 0, column: 0 };
            let _ = board.play(Coords { row: 1, column: 0 }, Color::Black);
            let _ = board.play(attacked_coords, Color::White);

            assert_eq!(
                board.find_attacked_bridges(attacked_coords),
                vec![Coords { row: 0, column: 1 },]
            );
        }

        #[test]
        fn test_bridge_to_own_edge() {
            //  a  b  c  d  e
            // 1\.  .  ○  .  .\1
            //  2\.  .  ●  .  .\2
            let mut board = Board::new(5);
            let attacked_coords = Coords { row: 0, column: 2 };
            let _ = board.play(Coords { row: 1, column: 2 }, Color::Black);
            let _ = board.play(attacked_coords, Color::White);

            assert_eq!(
                board.find_attacked_bridges(attacked_coords),
                vec![Coords { row: 0, column: 3 },]
            );
        }

        #[test]
        fn test_no_bridge_on_other_players_edge() {
            //  a  b  c  d  e
            // 1\.  .  .  .  .\1
            //  2\.  .  .  .  .\2
            //   3\○  ●  .  .  .\3
            let mut board = Board::new(5);
            let attacked_coords = Coords { row: 2, column: 0 };
            let _ = board.play(Coords { row: 2, column: 1 }, Color::Black);
            let _ = board.play(attacked_coords, Color::White);

            assert_eq!(board.find_attacked_bridges(attacked_coords), vec![]);
        }
    }
}
