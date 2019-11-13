use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

use crate::direction::Direction;
use crate::level::Level;
use crate::{Context2D, Point, SpriteSheet};

use super::board::Layer;
use super::SuccessState;

mod cell_cursor;
mod cell_type;
mod colour;
mod surroundings;

pub use cell_cursor::{CellCursor, CellCursorEntry, CellPalette, PaletteResult};
pub use cell_type::CellType;
pub use colour::Colour;
use surroundings::Surroundings;

pub trait Cell: Sized {
    /// If both cells are equal, set them to show the correct graphics.
    #[allow(clippy::mem_discriminant_non_enum)]
    fn calculate_surround(&mut self, other: &mut Self, direction: Direction) {
        let is_adjacent = std::mem::discriminant(self) == std::mem::discriminant(other);
        self.set_surround(direction, is_adjacent);
        other.set_surround(direction.opposite(), is_adjacent);
    }
    /// tell a cell that it's neghbor is of the same / different type.
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool);

    /// Returns the x,y coordinate of the given sprite on a spritesheet
    fn get_sprite_sheet_index(&self) -> Option<Point<u8>>;

    fn draw_into_layer(&self, layer: &mut Layer) {
        layer.add_cell(self.get_sprite_sheet_index());
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OverworldCell {
    Empty,
    Fence(Surroundings),
    Wall(Surroundings),
    BlockedPath(Surroundings),
    ClearPath(Surroundings),
}
impl Cell for OverworldCell {
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool) {
        match *self {
            OverworldCell::Fence(ref mut surrounds) 
            | OverworldCell::Wall(ref mut surrounds) 
            | OverworldCell::BlockedPath(ref mut surrounds)
            | OverworldCell::ClearPath(ref mut surrounds)=> {
                surrounds.set_surround(direction, is_adjacent)
            },
            OverworldCell::Empty => {},
        }
    }
    fn get_sprite_sheet_index (&self) -> Option<Point<u8>> {
        match *self {
            OverworldCell::Fence(surrounds) => Some(Point((surrounds).into(), 14)),
            OverworldCell::Wall(surrounds) => Some(Point((surrounds).into(), 15)),
            OverworldCell::BlockedPath(surrounds) => Some(Point((surrounds).into(), 8)),
            OverworldCell::ClearPath(surrounds) => Some(Point((surrounds).into(), 9)),
            | OverworldCell::Empty => None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OverlayCell {
    Empty,
    Success(Surroundings),
    Failure(Surroundings),
    Input(Surroundings),
    Output(Surroundings),
}
impl Cell for OverlayCell {
    fn get_sprite_sheet_index(&self) -> Option<Point<u8>> {
        match self {
            OverlayCell::Empty => None,
            OverlayCell::Success(surrounds) => Some(Point((*surrounds).into(), 13)),
            OverlayCell::Failure(surrounds) => Some(Point((*surrounds).into(), 12)),
            OverlayCell::Input(surrounds) => Some(Point((*surrounds).into(), 11)),
            OverlayCell::Output(surrounds) => Some(Point((*surrounds).into(), 10)),
        }
    }
    fn set_surround(&mut self, direction: Direction, is_adjacent: bool) {
        match *self {
            OverlayCell::Success(ref mut surrounds) 
            | OverlayCell::Failure(ref mut surrounds) 
            | OverlayCell::Input(ref mut surrounds)
            | OverlayCell::Output(ref mut surrounds)=> {
                surrounds.set_surround(direction, is_adjacent)
            },
            OverlayCell::Empty => {}
        }
    }
}
impl TryFrom<PaletteResult<CellType>> for OverlayCell {
    type Error = ();
    fn try_from(PaletteResult(cell_type, colour, direction): PaletteResult<CellType>) -> Result<Self, ()> {
        match (cell_type, colour) {
            (CellType::Overlay, Colour::Green) => Ok(OverlayCell::Success(Surroundings::new())),
            (CellType::Overlay, Colour::Red) => Ok(OverlayCell::Failure(Surroundings::new())),
            (CellType::Overlay, Colour::Orange) => Ok(OverlayCell::Input(Surroundings::new())),
            (CellType::Overlay, Colour::Blue) => Ok(OverlayCell::Output(Surroundings::new())),
            (CellType::Empty, _) => Ok(OverlayCell::Empty),
            _ => Err(()),
        }
    }
}
impl OverlayCell {
    pub fn success_state(&self) -> SuccessState {
        match self {
            OverlayCell::Success(_) => SuccessState::Succeeded,
            OverlayCell::Failure(_) => SuccessState::Failed,
            OverlayCell::Empty 
            | OverlayCell::Input(_)
            | OverlayCell::Output(_) => SuccessState::Running,
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
                let x = Into::<u8>::into(*colour) * 4 + Into::<u8>::into(*direction);
                Some(Point(x, 3))
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
impl TryFrom<PaletteResult<CellType>> for GroundCell {
    type Error = ();
    fn try_from(PaletteResult(cell_type, colour, direction): PaletteResult<CellType>) -> Result<Self, ()> {
        match cell_type {
            CellType::Empty => Ok(GroundCell::Empty),
            CellType::ColouredBlock => Ok(GroundCell::ColouredBlock(colour)),
            CellType::Arrow => Ok(GroundCell::Arrow(direction)),
            CellType::ColouredArrow => {
                Ok(GroundCell::ColouredArrow(colour, direction))
            }
            CellType::ArrowBlock => Ok(GroundCell::ArrowBlock(direction)),
            CellType::RotateLeft => Ok(GroundCell::RotateLeft),
            CellType::RotateRight => Ok(GroundCell::RotateRight),
            CellType::Fence => Ok(GroundCell::Fence(Surroundings::new())),
            CellType::Wall => Ok(GroundCell::Wall(Surroundings::new())),
            CellType::Overlay => Err(()),
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
