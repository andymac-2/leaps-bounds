use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::direction::Direction;
use crate::{Context2D, Point, SpriteSheet};

use super::board::Layer;
use super::SuccessState;

mod cell_cursor;
mod cell_type;
mod colour;
mod surroundings;

pub use cell_cursor::CellCursor;
use cell_type::CellType;
pub use colour::Colour;
use surroundings::Surroundings;

pub trait Cell: Sized {
    /// If both cells are equal, set them to show the correct graphics.
    #[allow(clippy::mem_discriminant_non_enum)]
    fn calculate_surround(&mut self, other: &mut Self, direction: Direction) {
        if std::mem::discriminant(self) == std::mem::discriminant(other) {
            self.set_surround(direction, true);
            other.set_surround(direction.opposite(), true);
        }
    }
    /// tell a cell that it's neghbor is of the same / different type.
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool);

    /// Returns the x,y coordinate of the given sprite on a spritesheet
    fn get_sprite_sheet_index(&self) -> Option<Point<u8>>;

    fn draw_into_layer(&self, layer: &mut Layer) {
        layer.add_cell(self.get_sprite_sheet_index());
    }
    fn draw(&self, context: &Context2D, sprite_sheet: &SpriteSheet) {
        if let Some(index) = self.get_sprite_sheet_index() {
            sprite_sheet.draw(context, index, Point(5.0, 5.0));
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OverlayCell {
    Empty,
    Success(Surroundings),
    Failure(Surroundings),
}
impl Cell for OverlayCell {
    fn get_sprite_sheet_index(&self) -> Option<Point<u8>> {
        match self {
            OverlayCell::Empty => None,
            OverlayCell::Success(surrounds) => Some(Point((*surrounds).into(), 13)),
            OverlayCell::Failure(surrounds) => Some(Point((*surrounds).into(), 12)),
        }
    }
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool) {
        match *self {
            OverlayCell::Success(ref mut surrounds) | OverlayCell::Failure(ref mut surrounds) => {
                surrounds.set_surround(direction, is_adjacent)
            }
            _ => {}
        }
    }
}
impl TryFrom<CellCursor> for OverlayCell {
    type Error = ();
    fn try_from(cell_cursor: CellCursor) -> Result<Self, ()> {
        match cell_cursor {
            CellCursor(CellType::Success, _, _) => Ok(OverlayCell::Success(Surroundings::new())),
            CellCursor(CellType::Failure, _, _) => Ok(OverlayCell::Failure(Surroundings::new())),
            _ => Err(()),
        }
    }
}
impl OverlayCell {
    pub fn success_state(&self) -> SuccessState {
        match self {
            OverlayCell::Success(_) => SuccessState::Succeeded,
            OverlayCell::Failure(_) => SuccessState::Failed,
            OverlayCell::Empty => SuccessState::Running,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum GroundCell {
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
impl Cell for GroundCell {
    fn get_sprite_sheet_index(&self) -> Option<Point<u8>> {
        match self {
            GroundCell::Empty => None,
            GroundCell::ColouredBlock(colour) => Some(Point((*colour).into(), 0)),
            GroundCell::Arrow(direction) => Some(Point((*direction).into(), 7)),
            GroundCell::ColouredArrow(colour, direction) => {
                Some(Point((*direction).into(), Into::<u8>::into(*colour) + 8))
            }
            GroundCell::ArrowBlock(direction) => Some(Point((*direction).into(), 1)),
            GroundCell::RotateLeft => Some(Point(1, 2)),
            GroundCell::RotateRight => Some(Point(0, 2)),
            GroundCell::Fence(surrounds) => Some(Point((*surrounds).into(), 14)),
            GroundCell::Wall(surrounds) => Some(Point((*surrounds).into(), 15)),
        }
    }
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool) {
        match *self {
            GroundCell::Fence(ref mut surrounds) | GroundCell::Wall(ref mut surrounds) => {
                surrounds.set_surround(direction, is_adjacent)
            }
            _ => {}
        }
    }
}
impl TryFrom<CellCursor> for GroundCell {
    type Error = ();
    fn try_from(cell_cursor: CellCursor) -> Result<Self, ()> {
        match cell_cursor {
            CellCursor(CellType::Empty, _, _) => Ok(GroundCell::Empty),
            CellCursor(CellType::ColouredBlock, colour, _) => Ok(GroundCell::ColouredBlock(colour)),
            CellCursor(CellType::Arrow, _, direction) => Ok(GroundCell::Arrow(direction)),
            CellCursor(CellType::ColouredArrow, colour, direction) => {
                Ok(GroundCell::ColouredArrow(colour, direction))
            }
            CellCursor(CellType::ArrowBlock, _, direction) => Ok(GroundCell::ArrowBlock(direction)),
            CellCursor(CellType::RotateLeft, _, _) => Ok(GroundCell::RotateLeft),
            CellCursor(CellType::RotateRight, _, _) => Ok(GroundCell::RotateRight),
            CellCursor(CellType::Fence, _, _) => Ok(GroundCell::Fence(Surroundings::new())),
            CellCursor(CellType::Wall, _, _) => Ok(GroundCell::Wall(Surroundings::new())),
            CellCursor(CellType::Success, _, _) => Err(()),
            CellCursor(CellType::Failure, _, _) => Err(()),
        }
    }
}

impl GroundCell {
    pub fn rotate_right(self) -> Self {
        match self {
            cell @ GroundCell::Empty => cell,
            cell @ GroundCell::ColouredBlock(_) => cell,
            cell @ GroundCell::Fence(_) => cell,
            cell @ GroundCell::Wall(_) => cell,
            GroundCell::Arrow(direction) => GroundCell::Arrow(direction.increment()),
            GroundCell::ColouredArrow(colour, direction) => {
                GroundCell::ColouredArrow(colour, direction.increment())
            }
            GroundCell::ArrowBlock(direction) => GroundCell::ArrowBlock(direction.increment()),
            GroundCell::RotateLeft => GroundCell::RotateRight,
            GroundCell::RotateRight => GroundCell::RotateLeft,
        }
    }
    pub fn rotate_left(self) -> Self {
        match self {
            cell @ GroundCell::Empty => cell,
            cell @ GroundCell::ColouredBlock(_) => cell,
            cell @ GroundCell::Fence(_) => cell,
            cell @ GroundCell::Wall(_) => cell,
            GroundCell::Arrow(direction) => GroundCell::Arrow(direction.decrement()),
            GroundCell::ColouredArrow(colour, direction) => {
                GroundCell::ColouredArrow(colour, direction.decrement())
            }
            GroundCell::ArrowBlock(direction) => GroundCell::ArrowBlock(direction.decrement()),
            GroundCell::RotateLeft => GroundCell::RotateRight,
            GroundCell::RotateRight => GroundCell::RotateLeft,
        }
    }
    pub fn is_solid_to_cows(self) -> bool {
        match self {
            GroundCell::Fence(_) | GroundCell::Wall(_) => true,
            _ => false,
        }
    }
}
