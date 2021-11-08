use hexgame::board::{Color, Coords};
use hexgame::game::{Game, Status};
use std::env;
use std::io;
use std::io::Write;

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
    } else {
        Ok(DEFAULT_SIZE)
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

    read_coords(&mut io::stdin().lock(), game.board.size())
}

fn read_coords<Reader: io::BufRead>(
    reader: &mut Reader,
    board_size: u8,
) -> Result<Coords, io::Error> {
    let mut input = String::new();
    reader.read_line(&mut input).expect("Failed to read line");

    let splitted: Vec<&str> = input.trim().split(',').collect();

    if splitted.len() != 2 {
        return Err(invalid_input("Invalid coordinates. Try something like 2,3"));
    }

    let row = splitted[0]
        .parse::<u8>()
        .map_err(|e| invalid_input(&e.to_string()))?;
    let column = splitted[1]
        .parse::<u8>()
        .map_err(|e| invalid_input(&e.to_string()))?;

    if row >= board_size || column >= board_size {
        return Err(invalid_input(&format!(
            "Coordinates must be in range 0 - {}",
            board_size - 1
        )));
    }
    Ok(Coords { row, column })
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
        let mut input = io::BufReader::new("1,2".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_eq!(result.unwrap(), Coords { row: 1, column: 2 });
    }

    #[test]
    fn test_read_coords_with_invalid_format() {
        let mut input = io::BufReader::new("1-2".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "Invalid coordinates");
    }

    #[test]
    fn test_read_coords_with_invalid_digit() {
        let mut input = io::BufReader::new("a,b".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "invalid digit");
    }

    #[test]
    fn test_read_coords_with_row_out_of_bounds() {
        let mut input = io::BufReader::new("1,3".as_bytes());
        let result = read_coords(&mut input, 3);
        assert_contains(result.unwrap_err(), "must be in range 0 - 2");
    }
}
