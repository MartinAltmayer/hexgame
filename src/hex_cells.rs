use std::cell::Cell;

use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::edges::{CoordsOrEdge, Edge};
use crate::union_find::UnionFind;

/// This type is used internally to index into `HexCells`.
/// It can refer to a cell (like `Coords`) or to an edge (like `Edge`).
/// Both `HexCells` and `Index` are considered internal types and not exposed.
/// Typically, we convert to one of `Coords`, `Edge`, or `CoordsOrEdge` instead of exposing an index.
pub type Index = u16;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HexCell {
    color: Option<Color>,
    parent: Cell<Option<Index>>,
}

const EDGES: [Edge; 4] = [Edge::Left, Edge::Top, Edge::Right, Edge::Bottom];

#[derive(Clone)]
pub struct HexCells {
    pub size: CoordValue,
    // layout is a vector with format [normal cells using index=row*size + column; left, top, right, bottom]
    vector: Vec<HexCell>,
}

impl HexCells {
    pub fn new(size: CoordValue) -> Self {
        let item_count = (size as Index) * (size as Index) + 4;
        Self {
            size,
            vector: vec![HexCell::default(); item_count as usize],
        }
    }

    pub fn index_from_coords(&self, coords: Coords) -> Index {
        let Coords { row, column } = coords;
        debug_assert!(
            coords.is_on_board_with_size(self.size),
            "Coords {} out of bounds. Must be at most {}",
            coords,
            Coords::new(self.size - 1, self.size - 1),
        );

        (row as Index) * (self.size as Index) + (column as Index)
    }

    pub fn decode_index(&self, index: Index) -> CoordsOrEdge {
        let size = self.size as Index;
        let first_edge_index = self.index_from_edge(EDGES[0]);

        if index < first_edge_index {
            Coords {
                row: (index / size) as CoordValue,
                column: (index % size) as CoordValue,
            }
            .into()
        } else {
            EDGES[(index - first_edge_index) as usize].into()
        }
    }

    pub fn coords_from_index(&self, index: Index) -> Coords {
        match self.decode_index(index) {
            CoordsOrEdge::Coords(coords) => coords,
            _ => panic!("Index {} cannot be converted to Coords", index),
        }
    }

    pub fn index_from_edge(&self, edge: Edge) -> Index {
        let start = (self.size as Index) * (self.size as Index);
        match edge {
            Edge::Left => start,
            Edge::Top => start + 1,
            Edge::Right => start + 2,
            Edge::Bottom => start + 3,
        }
    }

    pub fn index_from_coords_or_edge(&self, coords_or_edge: CoordsOrEdge) -> Index {
        match coords_or_edge {
            CoordsOrEdge::Coords(coords) => self.index_from_coords(coords),
            CoordsOrEdge::Edge(edge) => self.index_from_edge(edge),
        }
    }

    #[allow(dead_code)]
    pub fn get_color_at_coords(&self, coords: Coords) -> Option<Color> {
        self.get_color_at_index(self.index_from_coords(coords))
    }

    pub fn get_color_at_index(&self, index: Index) -> Option<Color> {
        self.vector[index as usize].color
    }

    #[allow(dead_code)]
    pub fn set_color_at_coords(&mut self, coords: Coords, color: Color) {
        self.set_color_at_index(self.index_from_coords(coords), color)
    }

    pub fn set_color_at_index(&mut self, index: Index, color: Color) {
        self.vector[index as usize].color = Some(color);
    }

    fn get_parent_at_index(&self, index: Index) -> Option<Index> {
        self.vector[index as usize].parent.get()
    }

    fn set_parent_at_index(&self, index: Index, parent: Index) {
        self.vector[index as usize].parent.set(Some(parent));
    }
}

impl UnionFind<Index> for HexCells {
    fn get_parent(&self, item: Index) -> Option<Index> {
        self.get_parent_at_index(item)
    }

    fn set_parent(&self, index: Index, parent: Index) {
        self.set_parent_at_index(index, parent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let cells = HexCells::new(3);
        assert_eq!(cells.size, 3);
        assert_eq!(cells.get_color_at_coords(Coords::new(0, 0)), None);
    }

    #[test]
    fn test_index_from_coords() {
        let cells = HexCells::new(3);
        assert_eq!(cells.index_from_coords(Coords::new(1, 2)), 5);
    }

    #[test]
    fn test_index_from_edge() {
        let cells = HexCells::new(3);
        assert_eq!(cells.index_from_edge(EDGES[0]), 9);
        assert_eq!(cells.index_from_edge(EDGES[1]), 10);
        assert_eq!(cells.index_from_edge(EDGES[2]), 11);
        assert_eq!(cells.index_from_edge(EDGES[3]), 12);
    }

    #[test]
    fn test_decode_index() {
        let cells = HexCells::new(3);
        assert_eq!(
            cells.decode_index(0),
            CoordsOrEdge::Coords(Coords::new(0, 0))
        );
        assert_eq!(cells.decode_index(9), CoordsOrEdge::Edge(EDGES[0]));
        assert_eq!(cells.decode_index(12), CoordsOrEdge::Edge(EDGES[3]));
    }

    #[test]
    fn test_set_color_at_index() {
        let color = Color::Black;
        let mut cells = HexCells::new(3);
        cells.set_color_at_index(5, color);
        assert_eq!(cells.get_color_at_index(5), Some(color));
        assert_eq!(cells.get_color_at_coords(Coords::new(1, 2)), Some(color));
    }

    #[test]
    fn test_set_parent_at_index() {
        let parent: Index = 127;
        let cells = HexCells::new(3);
        cells.set_parent_at_index(5, parent);
        assert_eq!(cells.get_parent_at_index(5), Some(parent));
    }
}
