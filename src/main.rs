use hexgame::{Color, Coords, MAX_BOARD_SIZE, MIN_BOARD_SIZE};
use hexgame::{Game, Status};
use std::env;
use std::io;
use std::io::Write;
use std::str::FromStr;

const DEFAULT_SIZE: u8 = 9;

fn main() -> std::io::Result<()> {
    let size = read_size()?;

    let mut game = Game::new(size);
    println!("{}", &game.board);

    loop {
        match game.status {
            Status::Ongoing => {
                let coords = request_coords(&game)?;
                play(&mut game, coords)?;
                println!("{}", &game.board);
            }
            Status::Finished(color) => {
                println!("Game Over! The winner is {:?}", color);
                return Ok(());
            }
        }
    }
}

fn read_size() -> std::io::Result<u8> {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [] | [_] => Ok(DEFAULT_SIZE),
        [_, size] => size
            .parse::<u8>()
            .map_err(invalid_input)
            .and_then(check_size),
        _ => Err(invalid_input(
            "Expected at most one command line argument - the size of the board",
        )),
    }
}

fn check_size(size: u8) -> std::io::Result<u8> {
    if !(MIN_BOARD_SIZE..=MAX_BOARD_SIZE).contains(&size) {
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
        Color::BLACK => "BLACK",
        Color::WHITE => "WHITE",
    };
    print!(
        "{}: Please enter the coordinates for your next move: ",
        player
    );
    io::stdout().flush()?;

    read_coords(io::stdin().lock(), game.board.size())
}

fn read_coords<Reader: io::BufRead>(reader: Reader, board_size: u8) -> std::io::Result<Coords> {
    let input = reader
        .lines()
        .next()
        .ok_or_else(|| invalid_input("Failed to read line"))??;

    Coords::from_str(input.trim())
        .map_err(invalid_input)
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
    game.play(coords).map_err(invalid_input)
}

fn invalid_input(message: impl ToString) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.to_string())
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
