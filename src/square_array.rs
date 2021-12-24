use crate::coords::{CoordValue, Coords};

pub type Index = u16;

#[derive(Clone)]
pub struct SquareArray<T: Copy + Default> {
    pub size: CoordValue,
    items: Vec<T>,
}

impl<T: Copy + Default> SquareArray<T> {
    pub fn new(size: CoordValue) -> SquareArray<T> {
        let item_count = size * size;
        SquareArray {
            size,
            items: vec![T::default(); item_count as usize],
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

    pub fn at_index(&self, index: Index) -> T {
        self.items[index as usize]
    }

    pub fn at_coord(&self, coords: Coords) -> T {
        self.at_index(self.index_from_coords(coords))
    }

    pub fn set_index(&mut self, index: Index, value: T) {
        self.items[index as usize] = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor() {
        let array: SquareArray<u16> = SquareArray::new(3);
        assert_eq!(array.size, 3);
        assert_eq!(array.at_coord(Coords::new(0, 0)), 0);
    }

    #[test]
    fn test_set_index() {
        let value = 123;
        let mut array: SquareArray<u16> = SquareArray::new(3);
        array.set_index(5, value);
        assert_eq!(array.at_index(5), value);
        assert_eq!(array.at_coord(Coords::new(1, 2)), value);
    }

    #[test]
    fn test_index_from_coords() {
        let array: SquareArray<u16> = SquareArray::new(3);
        assert_eq!(array.index_from_coords(Coords::new(1, 2)), 5);
    }
}
