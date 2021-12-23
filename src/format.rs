use std::fmt;

use crate::board::{Board, Color};
use crate::coords::{to_column_char, Coords};

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_column_labels(f, self.size(), 0)?;

        for row in 0..self.size() {
            write_row(f, self, row)?;
        }

        write_column_labels(f, self.size(), self.size() + 1)
    }
}

fn write_column_labels(f: &mut fmt::Formatter, board_size: u8, indent: u8) -> fmt::Result {
    write_indent(f, indent)?;

    for column in 0..board_size {
        write!(f, " {} ", to_column_char(column))?;
    }

    writeln!(f)
}

fn write_row(f: &mut fmt::Formatter, board: &Board, row: u8) -> fmt::Result {
    write_indent(f, row)?;
    write!(f, "{}\\", row + 1)?;

    for column in 0..board.size() {
        if column > 0 {
            write!(f, "  ")?;
        }
        let color = board.get_color(Coords { row, column });
        write!(f, "{}", char_for_color(color))?;
    }

    writeln!(f, "\\{}", row + 1)
}

fn write_indent(f: &mut fmt::Formatter, length: u8) -> fmt::Result {
    write!(f, "{}", " ".repeat(length.into()))
}

fn char_for_color(color: Option<Color>) -> char {
    match color {
        // Unfortunately, the Unicode hexagon characters ⬢ and ⬡
        // are displayed as 1,5-width in the terminals that I've tried.
        Some(Color::BLACK) => '●',
        Some(Color::WHITE) => '○',
        None => '.',
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
            " a  b  c \n",
            "1\\●  .  .\\1\n",
            " 2\\.  .  .\\2\n",
            "  3\\.  ○  .\\3\n",
            "     a  b  c \n",
        ));
    }
}
