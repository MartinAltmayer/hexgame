use std::fmt;

use crate::board::Board;
use crate::color::Color;
use crate::coords::{to_column_char, CoordValue, Coords};

impl fmt::Display for Board {
    /// Pretty human-readable format for boards.
    ///
    /// Example:
    /// ```text
    ///  a  b  c  d  e
    /// 1\.  .  .  .  .\1
    ///  2\.  ●  .  ○  .\2
    ///   3\.  .  ●  .  .\3
    ///    4\.  .  .  ○  .\4
    ///     5\.  .  .  .  .\5
    ///        a  b  c  d  e
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write_column_labels(f, self.size(), 0)?;

        for row in 0..self.size() {
            write_row(f, self, row)?;
        }

        write_column_labels(f, self.size(), (self.size() + 1) as usize)
    }
}

fn write_column_labels(
    f: &mut fmt::Formatter,
    board_size: CoordValue,
    indent: usize,
) -> fmt::Result {
    write_indent(f, indent)?;

    for column in 0..board_size {
        write!(f, " {} ", to_column_char(column))?;
    }

    writeln!(f)
}

fn write_row(f: &mut fmt::Formatter, board: &Board, row: CoordValue) -> fmt::Result {
    write_indent(f, row as usize)?;
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

fn write_indent(f: &mut fmt::Formatter, length: usize) -> fmt::Result {
    write!(f, "{}", " ".repeat(length))
}

fn char_for_color(color: Option<Color>) -> char {
    match color {
        // Unfortunately, the Unicode hexagon characters ⬢ and ⬡
        // are displayed as 1,5-width in the terminals that I've tried.
        Some(Color::Black) => '●',
        Some(Color::White) => '○',
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
        board.play(Coords { row: 0, column: 0 }, Color::Black).ok();
        board.play(Coords { row: 2, column: 1 }, Color::White).ok();

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
