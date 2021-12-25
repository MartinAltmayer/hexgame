use crate::color::Color;
use crate::coords::{CoordValue, Coords};

pub type Index = u16;

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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub color: Option<Color>,
    pub parent: Option<Position>,
}

#[derive(Clone)]
pub struct Cells {
    pub size: CoordValue,
    vector: Vec<Cell>,
}

impl Cells {
    pub fn new(size: CoordValue) -> Self {
        let item_count = size * size;
        Self {
            size,
            vector: vec![Cell::default(); item_count as usize],
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

    pub fn at_index(&self, index: Index) -> Cell {
        self.vector[index as usize]
    }

    pub fn at_coord(&self, coords: Coords) -> Cell {
        self.at_index(self.index_from_coords(coords))
    }

    pub fn set_index(&mut self, index: Index, value: Cell) {
        self.vector[index as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let array = Cells::new(3);
        assert_eq!(array.size, 3);
        assert_eq!(array.at_coord(Coords::new(0, 0)), Cell::default());
    }

    #[test]
    fn test_set_index() {
        let cell = Cell {
            color: Some(Color::Black),
            parent: None,
        };
        let mut array = Cells::new(3);
        array.set_index(5, cell);
        assert_eq!(array.at_index(5), cell);
        assert_eq!(array.at_coord(Coords::new(1, 2)), cell);
    }

    #[test]
    fn test_index_from_coords() {
        let array = Cells::new(3);
        assert_eq!(array.index_from_coords(Coords::new(1, 2)), 5);
    }
}
