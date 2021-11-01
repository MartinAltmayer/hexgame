use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Coords {
    pub row: u8,
    pub column: u8,
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {})", self.row, self.column)
    }
}

pub struct SquareArray<T: Default + Copy> {
    pub size: u8,
    items: Vec<T>,
}

impl<T: Default + Copy> SquareArray<T> {
    pub fn new(size: u8) -> SquareArray<T> {
        let item_count: usize = usize::from(size) * usize::from(size);
        SquareArray {
            size,
            items: vec![T::default(); item_count],
        }
    }

    pub fn index_from_coord(&self, coords: Coords) -> u16 {
        let Coords { row, column } = coords;
        debug_assert!(
            coords.row < self.size,
            "Row {} out of bounds. Must be at most {}",
            row,
            self.size - 1
        );
        debug_assert!(
            column < self.size,
            "Column {} out of bounds. Must be at most {}",
            column,
            self.size - 1
        );

        u16::from(row) * u16::from(self.size) + u16::from(column)
    }

    pub fn at_index(&self, index: u16) -> T {
        self.items[usize::from(index)]
    }

    pub fn at_coord(&self, coords: Coords) -> T {
        self.at_index(self.index_from_coord(coords))
    }

    pub fn set_index(&mut self, index: u16, value: T) {
        self.items[usize::from(index)] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let array: SquareArray<u16> = SquareArray::new(3);
        assert_eq!(array.size, 3);
        assert_eq!(array.at_coord(Coords { row: 0, column: 0 }), 0);
    }

    #[test]
    fn test_set_index() {
        let value = 123;
        let mut array: SquareArray<u16> = SquareArray::new(3);
        array.set_index(5, value);
        assert_eq!(array.at_index(5), value);
        assert_eq!(array.at_coord(Coords { row: 1, column: 2 }), value);
    }
}
