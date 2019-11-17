use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Colour {
    Red = 0,
    Blue = 1,
    Green = 2,
    Orange = 3,
}
impl Colour {
    pub const TOTAL_COLOURS: u8 = 4;
    pub fn increment(self) -> Self {
        match self {
            Colour::Red => Colour::Blue,
            Colour::Blue => Colour::Green,
            Colour::Green => Colour::Orange,
            Colour::Orange => Colour::Red,
        }
    }
}
impl Default for Colour {
    fn default() -> Self {
        Colour::Red
    }
}
impl From<Colour> for u8 {
    fn from(colour: Colour) -> Self {
        colour as u8
    }
}
