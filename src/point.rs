use crate::cell::Direction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Point<T>(pub T, pub T);

impl<T: Copy> Point<T> {
    pub fn x(&self) -> T {
        self.0
    }
    pub fn y(&self) -> T {
        self.1
    }
}
impl Point<i32> {
    pub fn increment_2d(&mut self, direction: Direction) {
        match direction {
            Direction::Up => self.1 -= 1,
            Direction::Right => self.0 += 1,
            Direction::Down => self.1 += 1,
            Direction::Left => self.0 -= 1, 
        }
    }
}
