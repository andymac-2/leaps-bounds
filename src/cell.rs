use crate::board::Layer;
use crate::{Context2D, Point, SpriteSheet};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
impl Direction {
    fn increment(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
    fn decrement(self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Right => Direction::Up,
            Direction::Down => Direction::Right,
            Direction::Left => Direction::Down,
        }
    }
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}
impl Into<u8> for Direction {
    fn into(self) -> u8 {
        match self {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Colour {
    Red,
    Blue,
    Green,
    Orange,
}
impl Colour {
    fn increment(self) -> Self {
        match self {
            Colour::Red => Colour::Blue,
            Colour::Blue => Colour::Green,
            Colour::Green => Colour::Orange,
            Colour::Orange => Colour::Red,
        }
    }
}
impl Into<u8> for Colour {
    fn into(self) -> u8 {
        match self {
            Colour::Red => 0,
            Colour::Blue => 1,
            Colour::Green => 2,
            Colour::Orange => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct Surroundings(u8);
impl Surroundings {
    fn new() -> Self {
        Surroundings(0)
    }
    fn set_surround(&mut self, direction: Direction, value: bool) {
        let bit: u8 = direction.into();
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CellType {
    Empty,
    ColouredBlock,
    Arrow,
    ColouredArrow,
    ArrowBlock,
    RotateRight,
    RotateLeft,
    Fence,
    Wall,
}
impl CellType {
    pub fn increment(self) -> Self {
        match self {
            CellType::Empty => CellType::ColouredBlock,
            CellType::ColouredBlock => CellType::Arrow,
            CellType::Arrow => CellType::ColouredArrow,
            CellType::ColouredArrow => CellType::ArrowBlock,
            CellType::ArrowBlock => CellType::RotateRight,
            CellType::RotateRight => CellType::RotateLeft,
            CellType::RotateLeft => CellType::Fence,
            CellType::Fence => CellType::Wall,
            CellType::Wall => CellType::Empty,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CellCursor(pub CellType, pub Colour, pub Direction);
impl From<Cell> for CellCursor {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Empty => CellCursor(CellType::Empty, Colour::Red, Direction::Up),
            Cell::ColouredBlock(colour) => {
                CellCursor(CellType::ColouredBlock, colour, Direction::Up)
            }
            Cell::Arrow(direction) => CellCursor(CellType::Arrow, Colour::Red, direction),
            Cell::ColouredArrow(colour, direction) => {
                CellCursor(CellType::ColouredArrow, colour, direction)
            }
            Cell::ArrowBlock(direction) => CellCursor(CellType::ArrowBlock, Colour::Red, direction),
            Cell::RotateRight => CellCursor(CellType::RotateRight, Colour::Red, Direction::Up),
            Cell::RotateLeft => CellCursor(CellType::RotateLeft, Colour::Red, Direction::Up),
            Cell::Fence(_) => CellCursor(CellType::Fence, Colour::Red, Direction::Up),
            Cell::Wall(_) => CellCursor(CellType::Wall, Colour::Red, Direction::Up),
        }
    }
}
impl CellCursor {
    pub fn new() -> Self {
        CellCursor(CellType::Empty, Colour::Red, Direction::Up)
    }
    pub fn increment_type(&mut self) {
        self.0 = self.0.increment();
    }
    pub fn increment_colour(&mut self) {
        self.1 = self.1.increment();
    }
    pub fn increment_direction(&mut self) {
        self.2 = self.2.increment();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Cell {
    Empty,
    ColouredBlock(Colour),
    Arrow(Direction),
    ColouredArrow(Colour, Direction),
    ArrowBlock(Direction),
    RotateRight,
    RotateLeft,
    Fence(Surroundings),
    Wall(Surroundings),
}
impl From<CellCursor> for Cell {
    fn from(cell_cursor: CellCursor) -> Self {
        match cell_cursor {
            CellCursor(CellType::Empty, _, _) => Cell::Empty,
            CellCursor(CellType::ColouredBlock, colour, _) => Cell::ColouredBlock(colour),
            CellCursor(CellType::Arrow, _, direction) => Cell::Arrow(direction),
            CellCursor(CellType::ColouredArrow, colour, direction) => {
                Cell::ColouredArrow(colour, direction)
            }
            CellCursor(CellType::ArrowBlock, _, direction) => Cell::ArrowBlock(direction),
            CellCursor(CellType::RotateLeft, _, _) => Cell::RotateLeft,
            CellCursor(CellType::RotateRight, _, _) => Cell::RotateRight,
            CellCursor(CellType::Fence, _, _) => Cell::Fence(Surroundings::new()),
            CellCursor(CellType::Wall, _, _) => Cell::Wall(Surroundings::new()),
        }
    }
}

impl Cell {
    fn get_sprite_sheet_index(self) -> Option<Point<u8>> {
        match self {
            Cell::Empty => None,
            Cell::ColouredBlock(colour) => Some(Point(colour.into(), 0)),
            Cell::Arrow(direction) => Some(Point(direction.into(), 7)),
            Cell::ColouredArrow(colour, direction) => {
                Some(Point(direction.into(), Into::<u8>::into(colour) + 8))
            }
            Cell::ArrowBlock(direction) => Some(Point(direction.into(), 1)),
            Cell::RotateLeft => Some(Point(1, 2)),
            Cell::RotateRight => Some(Point(0, 2)),
            Cell::Fence(surrounds) => Some(Point(surrounds.into(), 14)),
            Cell::Wall(surrounds) => Some(Point(surrounds.into(), 15)),
        }
    }
    pub fn rotate_right(self) -> Self {
        match self {
            cell @ Cell::Empty => cell,
            cell @ Cell::ColouredBlock(_) => cell,
            cell @ Cell::Fence(_) => cell,
            cell @ Cell::Wall(_) => cell,
            Cell::Arrow(direction) => Cell::Arrow(direction.increment()),
            Cell::ColouredArrow(colour, direction) => {
                Cell::ColouredArrow(colour, direction.increment())
            }
            Cell::ArrowBlock(direction) => Cell::ArrowBlock(direction.increment()),
            Cell::RotateLeft => Cell::RotateRight,
            Cell::RotateRight => Cell::RotateLeft,
        }
    }
    pub fn rotate_left(self) -> Self {
        match self {
            cell @ Cell::Empty => cell,
            cell @ Cell::ColouredBlock(_) => cell,
            cell @ Cell::Fence(_) => cell,
            cell @ Cell::Wall(_) => cell,
            Cell::Arrow(direction) => Cell::Arrow(direction.decrement()),
            Cell::ColouredArrow(colour, direction) => {
                Cell::ColouredArrow(colour, direction.decrement())
            }
            Cell::ArrowBlock(direction) => Cell::ArrowBlock(direction.decrement()),
            Cell::RotateLeft => Cell::RotateRight,
            Cell::RotateRight => Cell::RotateLeft,
        }
    }
    pub fn calculate_surround(&mut self, other: &mut Cell, direction: Direction) {
        self.reset_surround(direction);
        other.reset_surround(direction.opposite());
        match (*self, *other) {
            (Cell::Fence(_), Cell::Fence(_))
            | (Cell::Wall(_), Cell::Wall(_)) => {
                self.set_surround(direction);
                other.set_surround(direction.opposite());
            },
            _ => {}
        }
    }
    pub fn is_solid_to_cows(&self) -> bool {
        match self {
            Cell::Fence(_) | Cell::Wall(_) => true,
            _ => false,
        }
    }

    fn reset_surround(&mut self, direction: Direction) {
        match *self {
            Cell::Fence(ref mut surrounds) => surrounds.set_surround(direction, false),
            Cell::Wall(ref mut surrounds) => surrounds.set_surround(direction, false),
            _ => {}
        }
    }
    fn set_surround(&mut self, direction: Direction) {
        match *self {
            Cell::Fence(ref mut surrounds) => surrounds.set_surround(direction, true),
            Cell::Wall(ref mut surrounds) => surrounds.set_surround(direction, true),
            _ => {}
        }
    }

    pub fn draw_into_layer(self, layer: &mut Layer) {
        layer.add_cell(self.get_sprite_sheet_index());
    }
    pub fn draw_cursor_cell(self, context: &Context2D, sprite_sheet: &SpriteSheet) {
        self.get_sprite_sheet_index().map(|index| {
            sprite_sheet.draw(context, index, Point(5.0, 5.0));
        });
    }
}
