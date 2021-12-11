use std::error;
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Coords {
    pub row: u8,
    pub column: u8,
}

impl Coords {
    pub fn new(row: u8, column: u8) -> Self {
        Self { row, column }
    }

    pub fn is_on_board_with_size(&self, size: u8) -> bool {
        self.row < size && self.column < size
    }
}

impl std::str::FromStr for Coords {
    type Err = ParseCoordsError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let column = string.chars().nth(0).and_then(parse_column_char);
        let row = string
            .get(1..)
            .and_then(|s| s.parse::<u8>().ok())
            .filter(|&row| 0 < row)
            .map(|row| row - 1);

        match row.zip(column) {
            Some((row, column)) => Ok(Coords { row, column }),
            None => Err(ParseCoordsError {
                description: format!("Invalid coordinates: {}", string),
            }),
        }
    }
}

pub fn parse_column_char(c: char) -> Option<u8> {
    if 'a' <= c && c <= 'z' {
        Some((c as u8) - ('a' as u8))
    } else {
        None
    }
}

pub fn to_column_char(column: u8) -> char {
    (('a' as u8) + column) as char
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", to_column_char(self.column), self.row + 1)
    }
}

#[derive(Debug)]
pub struct ParseCoordsError {
    description: String,
}

impl fmt::Display for ParseCoordsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.description)
    }
}

impl error::Error for ParseCoordsError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_to_string() {
        assert_eq!(Coords::new(0, 0).to_string(), "a1");
        assert_eq!(Coords::new(12, 5).to_string(), "f13");
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Coords::from_str("a1").unwrap(), Coords::new(0, 0));
        assert_eq!(Coords::from_str("f13").unwrap(), Coords::new(12, 5));
    }

    #[test]
    fn test_from_invalid_strings() {
        assert!(Coords::from_str("").is_err());
        assert!(Coords::from_str("abc").is_err());
        assert!(Coords::from_str("A2").is_err());
        assert!(Coords::from_str("a0").is_err());
        assert!(Coords::from_str("ä2").is_err());
    }
}
