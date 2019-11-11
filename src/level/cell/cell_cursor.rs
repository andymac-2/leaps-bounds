use std::convert::TryFrom;

use crate::direction::Direction;
use crate::{Context2D, SpriteSheet};

use super::{Cell, CellType, Colour, GroundCell, OverlayCell};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CellCursor(pub CellType, pub Colour, pub Direction);
impl From<GroundCell> for CellCursor {
    fn from(cell: GroundCell) -> Self {
        match cell {
            GroundCell::Empty => CellCursor(CellType::Empty, Colour::Red, Direction::Up),
            GroundCell::ColouredBlock(colour) => {
                CellCursor(CellType::ColouredBlock, colour, Direction::Up)
            }
            GroundCell::Arrow(direction) => CellCursor(CellType::Arrow, Colour::Red, direction),
            GroundCell::ColouredArrow(colour, direction) => {
                CellCursor(CellType::ColouredArrow, colour, direction)
            }
            GroundCell::ArrowBlock(direction) => {
                CellCursor(CellType::ArrowBlock, Colour::Red, direction)
            }
            GroundCell::RotateRight => {
                CellCursor(CellType::RotateRight, Colour::Red, Direction::Up)
            }
            GroundCell::RotateLeft => CellCursor(CellType::RotateLeft, Colour::Red, Direction::Up),
            GroundCell::Fence(_) => CellCursor(CellType::Fence, Colour::Red, Direction::Up),
            GroundCell::Wall(_) => CellCursor(CellType::Wall, Colour::Red, Direction::Up),
        }
    }
}
impl From<OverlayCell> for CellCursor {
    fn from(cell: OverlayCell) -> Self {
        let cell_type = match cell {
            OverlayCell::Empty => CellType::Empty,
            OverlayCell::Success(_) => CellType::Success,
            OverlayCell::Failure(_) => CellType::Failure,
        };
        CellCursor(cell_type, Colour::Red, Direction::Up)
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
    pub fn draw(self, context: &Context2D, sprite_sheet: &SpriteSheet) {
        if let Ok(cell) = GroundCell::try_from(self) {
            cell.draw(context, sprite_sheet)
        } else if let Ok(cell) = OverlayCell::try_from(self) {
            cell.draw(context, sprite_sheet)
        }
    }
}
