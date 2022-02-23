/// This enum represents the two players. The first player is always `Black`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    /// Needs to connect top to bottom. Always starts the game.
    Black,
    /// Needs to connect left to right.
    White,
}

impl Color {
    /// Return the color of the other player.
    pub fn opponent_color(&self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}
