use std::fmt;

use crate::board::{Board, Color};
use crate::coords::Coords;

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in 0..self.size() {
            write!(f, "{}", " ".repeat(row.into()))?;

            for column in 0..self.size() {
                let color = self.get_color(Coords { row, column });
                write!(f, "{} ", char_for_color(color))?;
            }

            if row < self.size() - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn char_for_color(color: Option<Color>) -> char {
    match color {
        Some(Color::BLACK) => '⬢',
        Some(Color::WHITE) => '⬡',
        None => '⋅',
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn test_write_board() {
        let mut board = Board::new(3);
        board.play(Coords { row: 0, column: 0 }, Color::BLACK).ok();
        board.play(Coords { row: 2, column: 1 }, Color::WHITE).ok();

        let mut output = String::new();
        write!(&mut output, "{}", &board).ok();

        #[rustfmt::skip]
        assert_eq!(output, concat!(
            "⬢ ⋅ ⋅ \n",
            " ⋅ ⋅ ⋅ \n",
            "  ⋅ ⬡ ⋅ ",
        ));
    }
}
