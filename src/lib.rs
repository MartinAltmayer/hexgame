mod board;
mod format;
mod game;
mod square_array;
mod union_find;

pub use crate::board::{Board, Color, Coords, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
pub use crate::game::{Game, Status};
