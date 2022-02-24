use crate::hex_cells::{HexCells, Index};
use std::iter;
use std::iter::Iterator;

pub fn get_neighbors(cells: &HexCells, index: Index) -> impl Iterator<Item = Index> {
    let size = cells.size as Index;
    let left_neighbor = if index % size == 0 {
        cells.left()
    } else {
        index - 1
    };

    let top_left_neighbor = if index < size {
        cells.top()
    } else {
        index - size
    };

    let top_right_neighbor = if index >= size && index % size < size - 1 {
        Some(index - size + 1)
    } else {
        None
    };

    let right_neighbor = if index % size == size - 1 {
        cells.right()
    } else {
        index + 1
    };

    let bottom_right_neighbor = if index >= size * (size - 1) {
        cells.bottom()
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

#[cfg(test)]
mod tests {
    use crate::{CoordValue, Coords};

    use super::*;

    fn check_neighbors(cells: &HexCells, row: CoordValue, column: CoordValue, expected: &[Index]) {
        let index = cells.index_from_coords(Coords { row, column });
        let neighbors: Vec<Index> = get_neighbors(cells, index).collect();
        assert_eq!(neighbors, expected);
    }

    #[test]
    fn test_neighbors_top_left_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 0, &[
            cells.left(),
            cells.top(),
            cells.index_from_coords(Coords::new(0, 1)),
            cells.index_from_coords(Coords::new(1, 0)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 1, &[
            cells.index_from_coords(Coords::new(0, 0)),
            cells.top(),
            cells.index_from_coords(Coords::new(0, 2)),
            cells.index_from_coords(Coords::new(1, 1)),
            cells.index_from_coords(Coords::new(1, 0)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 1, 0, &[
            cells.left(),
            cells.index_from_coords(Coords::new(0, 0)),
            cells.index_from_coords(Coords::new(0, 1)),
            cells.index_from_coords(Coords::new(1, 1)),
            cells.index_from_coords(Coords::new(2, 0)),
        ]);
    }

    #[test]
    fn test_neighbors_top_right_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 4, &[
            cells.index_from_coords(Coords::new(0, 3)),
            cells.top(),
            cells.right(),
            cells.index_from_coords(Coords::new(1, 4)),
            cells.index_from_coords(Coords::new(1, 3)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 3, &[
            cells.index_from_coords(Coords::new(0, 2)),
            cells.top(),
            cells.index_from_coords(Coords::new(0, 4)),
            cells.index_from_coords(Coords::new(1, 3)),
            cells.index_from_coords(Coords::new(1, 2)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 1, 4, &[
            cells.index_from_coords(Coords::new(1, 3)),
            cells.index_from_coords(Coords::new(0, 4)),
            cells.right(),
            cells.index_from_coords(Coords::new(2, 4)),
            cells.index_from_coords(Coords::new(2, 3)),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 0, &[
            cells.left(),
            cells.index_from_coords(Coords::new(3, 0)),
            cells.index_from_coords(Coords::new(3, 1)),
            cells.index_from_coords(Coords::new(4, 1)),
            cells.bottom(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 3, 0, &[
            cells.left(),
            cells.index_from_coords(Coords::new(2, 0)),
            cells.index_from_coords(Coords::new(2, 1)),
            cells.index_from_coords(Coords::new(3, 1)),
            cells.index_from_coords(Coords::new(4, 0)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 1, &[
            cells.index_from_coords(Coords::new(4, 0)),
            cells.index_from_coords(Coords::new(3, 1)),
            cells.index_from_coords(Coords::new(3, 2)),
            cells.index_from_coords(Coords::new(4, 2)),
            cells.bottom(),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 4, &[
            cells.index_from_coords(Coords::new(4, 3)),
            cells.index_from_coords(Coords::new(3, 4)),
            cells.right(),
            cells.bottom(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 3, 4, &[
            cells.index_from_coords(Coords::new(3, 3)),
            cells.index_from_coords(Coords::new(2, 4)),
            cells.right(),
            cells.index_from_coords(Coords::new(4, 4)),
            cells.index_from_coords(Coords::new(4, 3)),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 3, &[
            cells.index_from_coords(Coords::new(4, 2)),
            cells.index_from_coords(Coords::new(3, 3)),
            cells.index_from_coords(Coords::new(3, 4)),
            cells.index_from_coords(Coords::new(4, 4)),
            cells.bottom(),
        ]);
    }

    #[test]
    fn test_neighbors_center() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 2, 2, &[
            cells.index_from_coords(Coords::new(2, 1)),
            cells.index_from_coords(Coords::new(1, 2)),
            cells.index_from_coords(Coords::new(1, 3)),
            cells.index_from_coords(Coords::new(2, 3)),
            cells.index_from_coords(Coords::new(3, 2)),
            cells.index_from_coords(Coords::new(3, 1)),
        ]);
    }
}
