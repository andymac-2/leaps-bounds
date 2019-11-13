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
impl Into<u8> for Colour {
    fn into(self) -> u8 {
        self as u8
    }
}
