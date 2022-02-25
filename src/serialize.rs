use crate::color::Color;
use crate::coords::CoordValue;
use crate::game::Game;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{Error, ErrorKind, Result};

const VERSION: u8 = 1;

/// This trait is implemented by Game and can be used to serialize/deserialize Hex games to/from strings or JSON.
pub trait Serialization: Sized {
    /// Save this game as a Serde JSON value
    fn save_to_json(&self) -> serde_json::Value;
    /// Save this game to a JSON string.
    fn save_to_string(&self) -> String {
        self.save_to_json().to_string()
    }
    /// Load a game from a Serde JSON value.
    fn load_from_json(value: Value) -> Result<Self>;
    /// Load a game from a JSON string.
    fn load_from_str(string: &str) -> Result<Self> {
        let value: Value = serde_json::from_str(string)?;
        Self::load_from_json(value)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredGame {
    version: u8,
    size: CoordValue,
    current_player: u8,
    stones: Vec<Vec<u8>>,
}

impl Serialization for Game {
    fn save_to_json(&self) -> Value {
        let stored_game = StoredGame {
            version: VERSION,
            size: self.get_board().size(),
            current_player: serialize_color(&self.get_current_player()),
            stones: store_stone_matrix(&self.get_board().to_stone_matrix()),
        };

        serde_json::to_value(&stored_game).expect("Game serialization failed")
    }

    fn load_from_json(value: Value) -> Result<Self> {
        let stored_game: StoredGame = serde_json::from_value(value)?;

        if stored_game.version != VERSION {
            return Err(invalid_data(format!(
                "Unsupported version: {}",
                stored_game.version
            )));
        }

        let stones = load_stone_matrix(&stored_game.stones)?;
        let current_player = deserialize_color(&stored_game.current_player)?;

        Game::load(stones, current_player).map_err(invalid_data)
    }
}

fn store_stone_matrix(stones: &[Vec<Option<Color>>]) -> Vec<Vec<u8>> {
    stones.iter().map(store_row).collect()
}

#[allow(clippy::ptr_arg)]
fn store_row(row: &Vec<Option<Color>>) -> Vec<u8> {
    row.iter().map(serialize_color).collect()
}

fn load_stone_matrix(stones: &[Vec<u8>]) -> Result<Vec<Vec<Option<Color>>>> {
    stones.iter().map(load_row).collect()
}

#[allow(clippy::ptr_arg)]
fn load_row(row: &Vec<u8>) -> Result<Vec<Option<Color>>> {
    row.iter().map(deserialize_color).collect()
}

fn serialize_color(color: &Option<Color>) -> u8 {
    match color {
        None => 0,
        Some(Color::Black) => 1,
        Some(Color::White) => 2,
    }
}

fn deserialize_color(input: &u8) -> Result<Option<Color>> {
    match input {
        0 => Ok(None),
        1 => Ok(Some(Color::Black)),
        2 => Ok(Some(Color::White)),
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
                "version": VERSION,
                "size": 2,
                "currentPlayer": 1,
                "stones": [[0, 1], [2, 0]]
            })
        );
    }

    #[test]
    fn test_deserialize() {
        let data = json!({
            "version": VERSION,
            "size": 2,
            "currentPlayer": 1,
            "stones": [[0, 1], [2, 0]]
        });

        let game = Game::load_from_json(data).unwrap();

        assert_eq!(game.get_board().size(), 2);
        assert_eq!(game.get_current_player(), Some(Color::Black));
        assert_eq!(
            game.get_board().get_color(Coords { row: 0, column: 0 }),
            None
        );
        assert_eq!(
            game.get_board().get_color(Coords { row: 0, column: 1 }),
            Some(Color::Black)
        );
        assert_eq!(
            game.get_board().get_color(Coords { row: 1, column: 0 }),
            Some(Color::White)
        );
        assert_eq!(
            game.get_board().get_color(Coords { row: 1, column: 1 }),
            None
        );
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
            "version": VERSION,
            "size": 2,
            "currentPlayer": 2,
            "stones": [[1, 0], [0, 0]],
        });

        let game = Game::load_from_json(data).unwrap();

        assert_eq!(game.get_current_player(), Some(Color::White));
    }

    #[test]
    fn test_deserialize_without_current_player() {
        let data = json!({
            "version": VERSION,
            "size": 2,
            "currentPlayer": 0,
            "stones": [[1, 0], [0, 0]],
        });

        let result = Game::load_from_json(data);

        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_with_invalid_version() {
        let data = json!({
            "version": VERSION + 1,
            "size": 2,
            "currentPlayer": 0,
            "stones": [[1, 0], [0, 0]],
        });

        let result = Game::load_from_json(data);

        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_to_string_cycle() {
        let mut game = Game::new(3);
        game.play(Coords { row: 0, column: 1 }).unwrap();
        game.play(Coords { row: 1, column: 0 }).unwrap();
        game.play(Coords { row: 1, column: 1 }).unwrap();

        let string = game.save_to_string();
        let loaded_game = Game::load_from_str(&string).unwrap();

        assert_eq!(game.get_board().size(), loaded_game.get_board().size());
        assert_eq!(game.get_current_player(), loaded_game.get_current_player());
        assert_eq!(
            game.get_board().to_stone_matrix(),
            loaded_game.get_board().to_stone_matrix()
        );
    }
}
