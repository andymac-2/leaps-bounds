use crate::direction::Direction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Surroundings(u8);
impl Surroundings {
    pub fn new() -> Self {
        Surroundings(0)
    }
    pub fn set_surround(&mut self, direction: Direction, value: bool) {
        let bit = direction as u8;
        if value {
            self.0 |= 0x1 << bit;
        } else {
            self.0 &= !(0x1 << bit);
        }
    }
}
impl Into<u8> for Surroundings {
    fn into(self) -> u8 {
        assert!(self.0 <= 0x0F);
        self.0
    }
}
