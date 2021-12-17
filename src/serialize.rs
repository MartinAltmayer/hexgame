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
struct StoredGame {
    size: u8,
    current_player: u8,
    cells: Vec<Vec<Option<u8>>>,
}

impl Serialization for Game {
    fn save_to_json(&self) -> Value {
        let stored_game = StoredGame {
            size: self.board.size(),
            current_player: serialize_color(self.current_player),
            cells: store_cells(&self.board.to_cells()),
        };

        serde_json::to_value(&stored_game).expect("Game serialization failed")
    }

    fn load_from_json(value: Value) -> Result<Self> {
        let stored_game: StoredGame = serde_json::from_value(value)?;
        let cells = load_cells(&stored_game.cells)?;
        let board = Board::from_cells(cells).map_err(invalid_data)?;

        Ok(Game {
            board,
            current_player: deserialize_color(stored_game.current_player)?,
            status: Status::Ongoing, // TODO
        })
    }
}

fn store_cells(cells: &Vec<Vec<Option<Color>>>) -> Vec<Vec<Option<u8>>> {
    cells.iter().map(store_row).collect()
}

fn store_row(row: &Vec<Option<Color>>) -> Vec<Option<u8>> {
    row.iter().map(|&cell| cell.map(serialize_color)).collect()
}

fn load_cells(cells: &Vec<Vec<Option<u8>>>) -> Result<Vec<Vec<Option<Color>>>> {
    cells.iter().map(load_row).collect()
}

fn load_row(row: &Vec<Option<u8>>) -> Result<Vec<Option<Color>>> {
    row.iter()
        .map(|option| option.map(deserialize_color).transpose())
        .collect()
}

fn serialize_color(color: Color) -> u8 {
    match color {
        Color::BLACK => 0,
        Color::WHITE => 1,
    }
}

fn deserialize_color(input: u8) -> Result<Color> {
    match input {
        0 => Ok(Color::BLACK),
        1 => Ok(Color::WHITE),
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

        // TODO: camel case
        assert_eq!(
            data,
            json!({
                "size": 2,
                "current_player": 0,
                "cells": [[Value::Null, 0], [1, Value::Null]]
            })
        );
    }

    #[test]
    fn test_deserialize() {
        let data = json!({
            "size": 2,
            "current_player": 0,
            "cells": [[Value::Null, 0], [1, Value::Null]]
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

        assert_eq!(data["current_player"], 1);
    }

    #[test]
    fn test_deserialize_white_as_current_player() {
        let data = json!({
            "size": 2,
            "current_player": 1,
            "cells": [[0, Value::Null], [Value::Null, Value::Null]],
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
