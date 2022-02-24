use std::cell::Cell;

use crate::color::Color;
use crate::coords::{CoordValue, Coords};
use crate::union_find::UnionFind;

/// This type can be used to refer to a cell (like `Coords`) or to an edge (top/left/right/bottom).
///
/// The `Index` values for a given cell or edge depend on the memory layout used by `hexgame` and are not guaranteed to remain stable.
/// For this reason `Index` values should be treated as opaque.
///
/// TODO: Do we need to expose the `Index` type?
pub type Index = u16;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct HexCell {
    color: Option<Color>,
    parent: Cell<Option<Index>>,
}

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

    pub fn coords_from_index(&self, index: Index) -> Coords {
        let size = self.size as Index;
        if index >= self.left() {
            panic!("Index {} cannot be converted to Coords", index);
        }
        Coords {
            row: (index / size) as CoordValue,
            column: (index % size) as CoordValue,
        }
    }

    pub fn left(&self) -> Index {
        let size = self.size as Index;
        size * size
    }

    pub fn top(&self) -> Index {
        self.left() + 1
    }

    pub fn right(&self) -> Index {
        self.left() + 2
    }

    pub fn bottom(&self) -> Index {
        self.left() + 3
    }

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
