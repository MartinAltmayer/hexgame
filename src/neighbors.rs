use crate::edges::Edge;
use crate::hex_cells::{HexCells, Index};
use std::iter;
use std::iter::Iterator;

pub fn get_neighbors(cells: &HexCells, index: Index) -> impl Iterator<Item = Index> {
    let size = cells.size as Index;
    let left_neighbor = if index % size == 0 {
        cells.index_from_edge(Edge::Left)
    } else {
        index - 1
    };

    let top_left_neighbor = if index < size {
        cells.index_from_edge(Edge::Top)
    } else {
        index - size
    };

    let top_right_neighbor = if index >= size && index % size < size - 1 {
        Some(index - size + 1)
    } else {
        None
    };

    let right_neighbor = if index % size == size - 1 {
        cells.index_from_edge(Edge::Right)
    } else {
        index + 1
    };

    let bottom_right_neighbor = if index >= size * (size - 1) {
        cells.index_from_edge(Edge::Bottom)
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
    use crate::{edges::CoordsOrEdge, CoordValue, Coords};

    use super::*;

    fn check_neighbors(
        cells: &HexCells,
        row: CoordValue,
        column: CoordValue,
        expected: &[CoordsOrEdge],
    ) {
        let index = cells.index_from_coords(Coords { row, column });
        let expected_indexes: Vec<Index> = expected
            .iter()
            .map(|&x| cells.index_from_coords_or_edge(x))
            .collect();
        let neighbors: Vec<Index> = get_neighbors(cells, index).collect();
        assert_eq!(neighbors, expected_indexes);
    }

    #[test]
    fn test_neighbors_top_left_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 0, &[
            Edge::Left.into(),
            Edge::Top.into(),
            Coords::new(0, 1).into(),
            Coords::new(1, 0).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 1, &[
            Coords::new(0, 0).into(),
            Edge::Top.into(),
            Coords::new(0, 2).into(),
            Coords::new(1, 1).into(),
            Coords::new(1, 0).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 1, 0, &[
            Edge::Left.into(),
            Coords::new(0, 0).into(),
            Coords::new(0, 1).into(),
            Coords::new(1, 1).into(),
            Coords::new(2, 0).into(),
        ]);
    }

    #[test]
    fn test_neighbors_top_right_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 4, &[
            Coords::new(0, 3).into(),
            Edge::Top.into(),
            Edge::Right.into(),
            Coords::new(1, 4).into(),
            Coords::new(1, 3).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 0, 3, &[
            Coords::new(0, 2).into(),
            Edge::Top.into(),
            Coords::new(0, 4).into(),
            Coords::new(1, 3).into(),
            Coords::new(1, 2).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 1, 4, &[
            Coords::new(1, 3).into(),
            Coords::new(0, 4).into(),
            Edge::Right.into(),
            Coords::new(2, 4).into(),
            Coords::new(2, 3).into(),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_left_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 0, &[
            Edge::Left.into(),
            Coords::new(3, 0).into(),
            Coords::new(3, 1).into(),
            Coords::new(4, 1).into(),
            Edge::Bottom.into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 3, 0, &[
            Edge::Left.into(),
            Coords::new(2, 0).into(),
            Coords::new(2, 1).into(),
            Coords::new(3, 1).into(),
            Coords::new(4, 0).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 1, &[
            Coords::new(4, 0).into(),
            Coords::new(3, 1).into(),
            Coords::new(3, 2).into(),
            Coords::new(4, 2).into(),
            Edge::Bottom.into(),
        ]);
    }

    #[test]
    fn test_neighbors_bottom_right_corner() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 4, &[
            Coords::new(4, 3).into(),
            Coords::new(3, 4).into(),
            Edge::Right.into(),
            Edge::Bottom.into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 3, 4, &[
            Coords::new(3, 3).into(),
            Coords::new(2, 4).into(),
            Edge::Right.into(),
            Coords::new(4, 4).into(),
            Coords::new(4, 3).into(),
        ]);
        #[rustfmt::skip]
        check_neighbors(&cells, 4, 3, &[
            Coords::new(4, 2).into(),
            Coords::new(3, 3).into(),
            Coords::new(3, 4).into(),
            Coords::new(4, 4).into(),
            Edge::Bottom.into(),
        ]);
    }

    #[test]
    fn test_neighbors_center() {
        let cells = HexCells::new(5);
        #[rustfmt::skip]
        check_neighbors(&cells, 2, 2, &[
            Coords::new(2, 1).into(),
            Coords::new(1, 2).into(),
            Coords::new(1, 3).into(),
            Coords::new(2, 3).into(),
            Coords::new(3, 2).into(),
            Coords::new(3, 1).into(),
        ]);
    }
}
