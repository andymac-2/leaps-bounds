use crate::Point;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}
impl Direction {
    pub fn to_index(self) -> u32 {
        match self {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        }
    }
    fn increment(self) -> Self {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
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
    fn to_index(self) -> u32 {
        match self {
            Colour::Red => 0,
            Colour::Blue => 1,
            Colour::Green => 2,
            Colour::Orange => 3,
        }
    }
    fn increment(self) -> Self {
        match self {
            Colour::Red => Colour::Blue,
            Colour::Blue => Colour::Green,
            Colour::Green => Colour::Orange,
            Colour::Orange => Colour::Red,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CellType {
    Empty,
    ColouredBlock,
    Arrow,
    ColouredArrow,
    ArrowBlock,
}
impl CellType {
    pub fn increment(self) -> Self {
        match self {
            CellType::Empty => CellType::ColouredBlock,
            CellType::ColouredBlock => CellType::Arrow,
            CellType::Arrow => CellType::ColouredArrow,
            CellType::ColouredArrow => CellType::ArrowBlock,
            CellType::ArrowBlock => CellType::Empty,
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
        }
    }
}

impl Cell {
    fn get_sprite_sheet_index(self) -> Point<u32> {
        match self {
            Cell::Empty => Point(0, 6),
            Cell::ColouredBlock(colour) => Point(colour.to_index(), 0),
            Cell::Arrow(direction) => Point(direction.to_index(), 7),
            Cell::ColouredArrow(colour, direction) => {
                Point(direction.to_index(), colour.to_index() + 8)
            }
            Cell::ArrowBlock(direction) => Point(direction.to_index(), 1),
        }
    }
    pub fn draw_into_buffer(self, buffer: &mut Vec<u8>) {
        let sprite_index = self.get_sprite_sheet_index();
        buffer.push(sprite_index.x() as u8);
        buffer.push(sprite_index.y() as u8);
    }
}
