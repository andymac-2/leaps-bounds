use crate::direction::Direction;
use crate::util::interpolate;
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
impl<Rhs, T> std::ops::Mul<Point<Rhs>> for Point<T>
where
    T: std::ops::Mul<Rhs>,
    Rhs: Clone,
{
    type Output = Point<<T as std::ops::Mul<Rhs>>::Output>;
    fn mul(self, rhs: Point<Rhs>) -> Self::Output {
        Point(self.0 * rhs.0, self.1 * rhs.1)
    }
}

pub fn interpolate_2d(start: Point<i32>, end: Point<i32>, proportion: f64) -> Point<f64> {
    assert!(proportion >= 0.0 && proportion <= 1.0);
    let x = interpolate(start.x().into(), end.x().into(), proportion);
    let y = interpolate(start.y().into(), end.y().into(), proportion);
    Point(x, y)
}
