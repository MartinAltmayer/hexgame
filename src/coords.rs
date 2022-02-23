use std::error;
use std::fmt;

/// The type of a single coordinate (row or column).
pub type CoordValue = u8;

/// Coordinates of a single cell of the board.
/// `hexgame` uses a zero-based (row, column)-format analogous to matrix-indices.
///
/// The following diagram shows on the left the format used by `Coords` and on the right
/// the "c4" format similar to Chess that is commonly used in the literature.
/// Note that the order of row-index and column-index is swapped between both formats:
/// The marked cell has coordinates (1, 3) and d2, respectively.
///
/// ```text
///  0  1  2  3  4           a  b  c  d  e
/// 0\.  .  .  .  .\0       1\.  .  .  .  .\1
///  1\.  .  .  ●  .\1       2\.  .  .  ●  .\2
///   2\.  .  .  .  .\2       3\.  .  .  .  .\3
///    3\.  .  .  .  .\3       4\.  .  .  .  .\4
///     4\.  .  .  .  .\4       5\.  .  .  .  .\5
///        0  1  2  3  4           a  b  c  d  e
/// ```
///
/// The `from_str` and `to_string` methods can be used to convert between the formats.
///
/// ```
/// # use hexgame::Coords;
/// use std::str::FromStr;
/// let coords = Coords::new(7, 0);
/// // Note the different order!
/// assert_eq!(coords.to_string(), "a8");
///
/// let other_coords = Coords::from_str("a8").unwrap();
/// assert_eq!(coords, other_coords);
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Coords {
    /// Zero-based row index, counted from top to bottom.
    pub row: CoordValue,
    /// Zero-based column index, counted from left to right.
    pub column: CoordValue,
}

impl Coords {
    /// Create a new Coords instance. Watch out: Order of parameters is different from
    /// the commonly used "c4" format.
    pub fn new(row: CoordValue, column: CoordValue) -> Self {
        Self { row, column }
    }

    /// Return whether this coordinate exist on a board of the given size.
    pub fn is_on_board_with_size(&self, size: CoordValue) -> bool {
        self.row < size && self.column < size
    }
}

impl std::str::FromStr for Coords {
    type Err = ParseCoordsError;

    /// Parse a coordinate from "c4" format.
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let column = string.chars().next().and_then(parse_column_char);
        let row = string
            .get(1..)
            .and_then(|s| s.parse::<CoordValue>().ok())
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

/// Convert a character to a single coordinate (characters are used to denote column indices in the "c4" format).
/// Return None if `c` is not in the range a-z.
pub fn parse_column_char(c: char) -> Option<CoordValue> {
    if ('a'..='z').contains(&c) {
        Some(((c as u8) - b'a') as CoordValue)
    } else {
        None
    }
}

/// Convert a single coordinate (typically the column index) to the character that is used in the "c4" format.
pub fn to_column_char(column: CoordValue) -> char {
    (b'a' + (column as u8)) as char
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}{}", to_column_char(self.column), self.row + 1)
    }
}

/// Returned by `Coords::from_str` if the string cannot be parsed.
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
