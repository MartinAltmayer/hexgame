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

    pub fn index_from_coord(&self, row: u8, column: u8) -> u16 {
        if cfg!(debug_assertions) {
            if row >= self.size {
                panic!(
                    "Row {} out of bounds. Must be at most {}",
                    row,
                    self.size - 1
                );
            }
            if column >= self.size {
                panic!(
                    "Column {} out of bounds. Must be at most {}",
                    column,
                    self.size - 1
                );
            }
        }
        u16::from(row) * u16::from(self.size) + u16::from(column)
    }

    pub fn at_index(&self, index: u16) -> T {
        self.items[usize::from(index)]
    }

    pub fn at_coord(&self, row: u8, column: u8) -> T {
        self.at_index(self.index_from_coord(row, column))
    }

    pub fn set_index(&mut self, index: u16, value: T) {
        self.items[usize::from(index)] = value;
    }

    pub fn set_coord(&mut self, row: u8, column: u8, value: T) {
        let index = self.index_from_coord(row, column);
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
        assert_eq!(array.at_coord(0, 0), 0);
    }

    #[test]
    fn test_set_index() {
        let value = 123;
        let mut array: SquareArray<u16> = SquareArray::new(3);
        array.set_index(5, value);
        assert_eq!(array.at_index(5), value);
        assert_eq!(array.at_coord(1, 2), value);
    }

    #[test]
    fn test_set_coord() {
        let value = 123;
        let mut array: SquareArray<u16> = SquareArray::new(3);
        array.set_coord(2, 1, value);
        assert_eq!(array.at_coord(2, 1), value);
        assert_eq!(array.at_index(7), value);
    }
}
