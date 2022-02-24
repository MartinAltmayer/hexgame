use crate::attacked_bridges::find_attacked_bridges;
use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::edges::{set_edge_colors, CoordsOrEdge};
use crate::errors::{InvalidBoard, InvalidMove};
use crate::hex_cells::{HexCells, Index};
use crate::neighbors::get_neighbors;
use crate::union_find::UnionFind;
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
/// use hexgame::{Board, Color, Coords};
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
        let mut cells = HexCells::new(size);
        set_edge_colors(&mut cells);
        Self { cells }
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

    /// Return the color at the given coordinates or edge.
    /// If no stone has been placed in the given cell, this method will return None.
    pub fn get_color<T: Into<CoordsOrEdge>>(&self, coords_or_edge: T) -> Option<Color> {
        self.cells
            .get_color_at_index(self.cells.index_from_coords_or_edge(coords_or_edge.into()))
    }

    fn get_color_at_index(&self, index: Index) -> Option<Color> {
        self.cells.get_color_at_index(index)
    }

    pub fn is_in_same_set<S: Into<CoordsOrEdge>, T: Into<CoordsOrEdge>>(&self, s: S, t: T) -> bool {
        self.cells.is_in_same_set(
            self.cells.index_from_coords_or_edge(s.into()),
            self.cells.index_from_coords_or_edge(t.into()),
        )
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
        let mut iter = get_neighbors(&self.cells, index);

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

    /// Return all bridges that are attacked by a given stone.
    ///
    /// A bridge is the most common virtual connection pattern in Hex.
    /// In the following example, White cannot prevent Black from connecting their stones:
    /// If White places a stone between the black stones, Black may just choose the other cell between the black stones.
    /// ```text
    ///  a  b  c
    /// 1\.  .  .\1
    ///  2\.  .  ●\2
    ///   3\●  .  .\3
    ///      a  b  c
    /// ```
    ///
    /// This method assumes that one player has just played on `coords`.
    /// It computes all bridges of the other player that are being attacked by this move.
    /// For each attacked bridge, the free cell in the middle is returned.
    /// If the attacked player places a stone on this cell, the bridge will be fully connected despite the attack.
    ///
    /// This method will also return bridges that connect a stone to one of the player's own edges (see the example below).
    ///
    /// The color at `coords` determines the attacking player
    /// (if the cell at `coords` is empty, this method returns an empty list.)
    ///
    /// Example:
    /// ```text
    ///  a  b  c
    /// 1\.  .  .\1
    ///  2\.  .  ●\2
    ///   3\●  ○  .\3
    ///      a  b  c
    /// ```
    ///
    /// The attacked bridges on this board and coords=b3 are:
    ///
    /// - a3 - b2 - c2
    /// - c2 - c3 - bottom edge
    ///
    /// Thus, this method would return the middle cells [b2, c3].
    pub fn find_attacked_bridges(&self, coords: Coords) -> Vec<Coords> {
        find_attacked_bridges(&self.cells, coords)
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
    use crate::edges::Edge;

    use super::*;

    #[test]
    fn test_new() {
        let board = Board::new(3);
        assert_eq!(board.size(), 3);
        assert!(board.get_color(Coords { row: 0, column: 0 }).is_none());
        assert_eq!(board.get_color(Edge::Left), Some(Color::White));
        assert_eq!(board.get_color(Edge::Top), Some(Color::Black));
        assert_eq!(board.get_color(Edge::Right), Some(Color::White));
        assert_eq!(board.get_color(Edge::Bottom), Some(Color::Black));
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
        let neighbor1 = Coords { row: 1, column: 2 };
        let neighbor2 = Coords { row: 3, column: 2 };
        board.play(neighbor1, Color::Black).unwrap();
        board.play(neighbor2, Color::Black).unwrap();

        assert!(!board.is_in_same_set(neighbor1, neighbor2));

        board.play(center, Color::Black).unwrap();

        assert!(board.is_in_same_set(neighbor1, neighbor2));
        assert!(board.is_in_same_set(center, neighbor2));
    }

    #[test]
    fn test_do_not_merge_neighbors_of_other_color() {
        let mut board = Board::new(5);
        let center = Coords { row: 2, column: 2 };
        let neighbor = Coords { row: 1, column: 2 };

        board.play(neighbor, Color::White).unwrap();
        board.play(center, Color::Black).unwrap();

        assert!(!board.is_in_same_set(center, neighbor));
    }

    #[test]
    fn test_do_not_merge_cells_that_are_not_connected() {
        let mut board = Board::new(3);
        let top_left = Coords { row: 0, column: 0 };
        let bottom_right = Coords { row: 2, column: 2 };

        board.play(top_left, Color::Black).unwrap();
        board.play(bottom_right, Color::Black).unwrap();

        assert!(!board.is_in_same_set(top_left, bottom_right));
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
