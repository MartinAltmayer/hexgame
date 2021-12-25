use crate::color::Color;
use crate::coords::{CoordValue, Coords};

pub type Index = u16;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Cell {
    pub color: Option<Color>,
    pub parent: Option<Index>,
}

impl Cell {
    pub fn with_color(color: Color) -> Self {
        Self {
            color: Some(color),
            parent: None,
        }
    }
}

#[derive(Clone)]
pub struct Cells {
    pub size: CoordValue,
    // layout is [normal cells by index=row*size + column; left, top, right, bottom]
    vector: Vec<Cell>,
}

impl Cells {
    pub fn new(size: CoordValue) -> Self {
        let item_count = (size as Index) * (size as Index) + 4;
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

    pub fn at_index(&self, index: Index) -> Cell {
        self.vector[index as usize]
    }

    pub fn at_coord(&self, coords: Coords) -> Cell {
        self.at_index(self.index_from_coords(coords))
    }

    pub fn set_index(&mut self, index: Index, cell: Cell) {
        self.vector[index as usize] = cell;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let cells = Cells::new(3);
        assert_eq!(cells.size, 3);
        assert_eq!(cells.at_coord(Coords::new(0, 0)), Cell::default());
    }

    #[test]
    fn test_index_from_coords() {
        let cells = Cells::new(3);
        assert_eq!(cells.index_from_coords(Coords::new(1, 2)), 5);
    }

    #[test]
    fn test_set_index() {
        let cell = Cell {
            color: Some(Color::Black),
            parent: None,
        };
        let mut cells = Cells::new(3);
        cells.set_index(5, cell);
        assert_eq!(cells.at_index(5), cell);
        assert_eq!(cells.at_coord(Coords::new(1, 2)), cell);
    }
}
