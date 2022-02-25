use crate::coords::{CoordValue, Coords};
use std::error::Error;
use std::fmt;

/// This enum is returned by `game.play` when the given move is invalid.
#[derive(Debug, PartialEq)]
pub enum InvalidMove {
    /// The game has ended, thus no further move is possible.
    GameOver,
    /// The player attempted to play on coordinates that are not on the game's board.
    OutOfBounds(Coords),
    /// The player attempted to play on coordinates that are already occupied.
    CellOccupied(Coords),
}

impl fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            InvalidMove::GameOver => write!(f, "Game has ended"),
            InvalidMove::OutOfBounds(coords) => {
                write!(f, "Coordinates {} are out of bounds", coords)
            }
            InvalidMove::CellOccupied(coords) => {
                write!(f, "Cell {} is already occupied", coords)
            }
        }
    }
}

impl Error for InvalidMove {}

/// This error may be returned by methods that load a board from serialized data.
#[derive(Debug, PartialEq, Eq)]
pub enum InvalidBoard {
    /// The size of the serialized board is not supported. Board sizes must be bounded by `MIN_BOARD_SIZE` and `MAX_BOARD_SIZE`.
    /// The values contained in this error are: the size of the serialized data, the minimal supported size, and the maximal supported size.
    SizeOutOfBounds(usize, CoordValue, CoordValue),
    /// The serialized board is not square, i.e. at least one row has more/less entries than there are rows.
    /// The values contained in this error are: the detected board size (number of rows) and the index of the row where the problem occurred.
    NotSquare(CoordValue, CoordValue),
    /// When loading a game, no current player was specified, although the game has not yet finished.
    NoCurrentPlayer,
}

impl fmt::Display for InvalidBoard {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            InvalidBoard::SizeOutOfBounds(size, min, max) => write!(
                f,
                "Board size must be between {} and {}. Found {}",
                min, max, size
            ),
            InvalidBoard::NotSquare(size, row_index) => write!(
                f,
                "Length of row {} does not match board size {}",
                row_index, size
            ),
            InvalidBoard::NoCurrentPlayer => {
                write!(f, "No current player given, but game has not yet finished")
            }
        }
    }
}

impl Error for InvalidBoard {}
