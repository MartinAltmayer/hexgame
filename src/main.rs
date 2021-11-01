use hexgame::board::{Color, Coords};
use hexgame::game::Game;
use std::io;
use std::io::Write;

fn main() {
    let mut game = Game::new(3);
    println!("{}", &game.board);

    loop {
        let result = request_and_play_move(&mut game);

        match result {
            Ok(_) => {
                println!("{}", &game.board);
            }
            Err(error) => {
                println!("Error: {}", error);
            }
        }
    }
}

fn request_and_play_move(game: &mut Game) -> Result<(), io::Error> {
    let coords = request_coords(game)?;
    game.play(coords)
        .map_err(|error| invalid_input(&error.to_string()))
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
