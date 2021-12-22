use hexgame::{Color, Coords, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use hexgame::{Game, Status};
use std::env;
use std::io;
use std::io::Write;
use std::str::FromStr;

const DEFAULT_SIZE: u8 = 9;

fn main() {
    let size = match read_size() {
        Ok(size) => size,
        Err(error) => {
            println!("Error: {}", error);
            return;
        }
    };

    let mut game = Game::new(size);
    println!("{}", &game.board);

    loop {
        match game.status {
            Status::Ongoing => {
                let result = request_coords(&game).and_then(|coords| play(&mut game, coords));

                match result {
                    Ok(_) => {
                        println!("{}", &game.board);
                    }
                    Err(error) => {
                        println!("Error: {}", error);
                    }
                }
            }
            Status::Finished(color) => {
                println!("Game Over! The winner is {:?}", color);
                return;
            }
        }
    }
}

fn read_size() -> std::io::Result<u8> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        return Err(invalid_input(
            "Expected at most one command line argument - the size of the board",
        ));
    }

    if args.len() == 2 {
        args[1]
            .parse::<u8>()
            .map_err(|e| invalid_input(&e.to_string()))
            .and_then(check_size)
    } else {
        Ok(DEFAULT_SIZE)
    }
}

fn check_size(size: u8) -> std::io::Result<u8> {
    if size < MIN_BOARD_SIZE || size > MAX_BOARD_SIZE {
        Err(invalid_input(&format!(
            "Size must be between {} and {}",
            MIN_BOARD_SIZE, MAX_BOARD_SIZE
        )))
    } else {
        Ok(size)
    }
}

fn request_coords(game: &Game) -> Result<Coords, io::Error> {
    let player = match game.current_player {
        Color::Black => "BLACK",
        Color::White => "WHITE",
    };
    print!(
        "{}: Please enter the coordinates for your next move: ",
        player
    );
    io::stdout().flush()?;

    read_coords(&mut io::stdin().lock(), game.board.size())
}

fn read_coords<Reader: io::BufRead>(
    reader: &mut Reader,
    board_size: u8,
) -> Result<Coords, io::Error> {
    let mut input = String::new();
    reader.read_line(&mut input).expect("Failed to read line");

    Coords::from_str(input.trim())
        .map_err(|error| invalid_input(&error.to_string()))
        .and_then(|coords| {
            if coords.is_on_board_with_size(board_size) {
                Ok(coords)
            } else {
                Err(invalid_input(&format!(
                    "Coordinates must be in range {} to {}",
                    Coords::new(0, 0),
                    Coords::new(board_size - 1, board_size - 1)
                )))
            }
        })
}

fn play(game: &mut Game, coords: Coords) -> Result<(), io::Error> {
    game.play(coords)
        .map_err(|error| invalid_input(&error.to_string()))
}

fn invalid_input(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_contains(error: io::Error, substring: &str) {
        let message = error.to_string();
        assert!(
            message.contains(substring),
            "Message '{}' does not contain expected substring '{}'",
            message,
            substring
        );
    }

    #[test]
    fn test_read_coords_with_valid_coords() {
        let mut input = io::BufReader::new("c2".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_eq!(result.unwrap(), Coords { row: 1, column: 2 });
    }

    #[test]
    fn test_read_coords_with_invalid_format() {
        let mut input = io::BufReader::new("b-1".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "Invalid coordinates");
    }

    #[test]
    fn test_read_coords_with_row_out_of_bounds() {
        let mut input = io::BufReader::new("a4".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "must be in range a1 to c3");
    }

    #[test]
    fn test_read_coords_with_column_out_of_bounds() {
        let mut input = io::BufReader::new("d2".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "must be in range a1 to c3");
    }
}
