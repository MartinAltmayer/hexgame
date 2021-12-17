use crate::coords::Coords;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum InvalidMove {
    GameOver,
    OutOfBounds(Coords),
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

#[derive(Debug, PartialEq)]
pub enum InvalidBoard {
    SizeOutOfBounds(usize, u8, u8),
    NotSquare(u8, u8),
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
        }
    }
}

impl Error for InvalidBoard {}
