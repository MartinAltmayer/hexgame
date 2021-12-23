use crate::board::Board;
use crate::board::Color;
use crate::game::{Game, Status};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use std::io::{Error, ErrorKind, Result};

pub trait Serialization: Sized {
    fn save_to_json(&self) -> serde_json::Value;
    fn save_to_string(&self) -> String {
        self.save_to_json().to_string()
    }
    fn load_from_json(value: Value) -> Result<Self>;
    fn load_from_str(string: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(string)?;
        Self::load_from_json(value)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredGame {
    size: u8,
    current_player: u8,
    cells: Vec<Vec<u8>>,
}

impl Serialization for Game {
    fn save_to_json(&self) -> Value {
        let stored_game = StoredGame {
            size: self.board.size(),
            current_player: serialize_color(&Some(self.current_player)),
            cells: store_cells(&self.board.to_cells()),
        };

        serde_json::to_value(&stored_game).expect("Game serialization failed")
    }

    fn load_from_json(value: Value) -> Result<Self> {
        let stored_game: StoredGame = serde_json::from_value(value)?;
        let cells = load_cells(&stored_game.cells)?;
        let board = Board::from_cells(cells).map_err(invalid_data)?;
        let current_player = deserialize_color(&stored_game.current_player)?;
        let current_player = current_player.ok_or_else(|| invalid_data("Current player is 0"))?;

        Ok(Game {
            board,
            // current player doesn't matter if game has finished.
            current_player,
            status: Status::Ongoing, // TODO
        })
    }
}

fn store_cells(cells: &[Vec<Option<Color>>]) -> Vec<Vec<u8>> {
    cells.iter().map(|v| store_row(v)).collect()
}

fn store_row(row: &[Option<Color>]) -> Vec<u8> {
    row.iter().map(serialize_color).collect()
}

fn load_cells(cells: &[Vec<u8>]) -> Result<Vec<Vec<Option<Color>>>> {
    cells.iter().map(|v| load_row(v)).collect()
}

fn load_row(row: &[u8]) -> Result<Vec<Option<Color>>> {
    row.iter().map(deserialize_color).collect()
}

fn serialize_color(color: &Option<Color>) -> u8 {
    match color {
        None => 0,
        Some(Color::BLACK) => 1,
        Some(Color::WHITE) => 2,
    }
}

fn deserialize_color(input: &u8) -> Result<Option<Color>> {
    match input {
        0 => Ok(None),
        1 => Ok(Some(Color::BLACK)),
        2 => Ok(Some(Color::WHITE)),
        _ => Err(invalid_data(format!("Invalid color {}", input))),
    }
}

fn invalid_data<T: ToString>(message: T) -> Error {
    Error::new(ErrorKind::InvalidData, message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coords::Coords;
    use serde_json::json;

    #[test]
    fn test_serialize() {
        let mut game = Game::new(2);
        game.play(Coords { row: 0, column: 1 }).unwrap();
        game.play(Coords { row: 1, column: 0 }).unwrap();

        let data = game.save_to_json();

        assert_eq!(
            data,
            json!({
                "size": 2,
                "currentPlayer": 1,
                "cells": [[0, 1], [2, 0]]
            })
        );
    }

    #[test]
    fn test_deserialize() {
        let data = json!({
            "size": 2,
            "currentPlayer": 1,
            "cells": [[0, 1], [2, 0]]
        });

        let game = Game::load_from_json(data).unwrap();

        assert_eq!(game.board.size(), 2);
        assert_eq!(game.current_player, Color::BLACK);
        assert_eq!(game.board.get_color(Coords { row: 0, column: 0 }), None);
        assert_eq!(
            game.board.get_color(Coords { row: 0, column: 1 }),
            Some(Color::BLACK)
        );
        assert_eq!(
            game.board.get_color(Coords { row: 1, column: 0 }),
            Some(Color::WHITE)
        );
        assert_eq!(game.board.get_color(Coords { row: 1, column: 1 }), None);
    }

    #[test]
    fn test_serialize_white_as_current_player() {
        let mut game = Game::new(2);
        game.play(Coords { row: 0, column: 0 }).unwrap();

        let data = game.save_to_json();

        assert_eq!(data["currentPlayer"], 2);
    }

    #[test]
    fn test_deserialize_white_as_current_player() {
        let data = json!({
            "size": 2,
            "currentPlayer": 2,
            "cells": [[1, 0], [0, 0]],
        });

        let game = Game::load_from_json(data).unwrap();

        assert_eq!(game.current_player, Color::WHITE);
    }

    #[test]
    fn test_serialization_to_string_cycle() {
        let mut game = Game::new(3);
        game.play(Coords { row: 0, column: 1 }).unwrap();
        game.play(Coords { row: 1, column: 0 }).unwrap();
        game.play(Coords { row: 1, column: 1 }).unwrap();

        let string = game.save_to_string();
        let loaded_game = Game::load_from_str(&string).unwrap();

        assert_eq!(game.board.size(), loaded_game.board.size());
        assert_eq!(game.current_player, loaded_game.current_player);
        assert_eq!(game.board.to_cells(), loaded_game.board.to_cells());
    }
    // TODO: test errors?
}
