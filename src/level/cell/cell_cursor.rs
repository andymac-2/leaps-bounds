use std::convert::{TryFrom, TryInto};

use crate::console_log;
use crate::component::{Component, Translation, Rect};
use crate::point::Point;
use crate::direction::Direction;
use crate::util::with_saved_context;
use crate::{Assets, Context2D, SpriteSheet};

use super::{Cell, CellType, Colour, GroundCell, OverlayCell};

pub struct PaletteResult<T>(pub T, pub Colour, pub Direction);

#[derive(Debug, Clone)]
pub struct CellPalette<T> {
    palette: Translation<Palette<T>>,
    control: Translation<PaletteControl>,
    is_collapsed: bool,
}
impl<T> CellPalette<T> {
    const LEFT_MARGIN: i32 = SpriteSheet::STANDARD_WIDTH / 2;
    const TOP_MARGIN: i32 = SpriteSheet::STANDARD_HEIGHT / 2;
    const CONTROL_OFFSET: Point<i32> = Point(Self::LEFT_MARGIN, Self::TOP_MARGIN);
    const PALETTE_OFFSET: Point<i32> = Point(Self::LEFT_MARGIN, Self::TOP_MARGIN * 4);

    pub fn new(palette: Vec<CellCursorEntry<T>>) -> Self {
        assert!(palette.len() >= 1);
        CellPalette {
            palette: Translation::new(Self::PALETTE_OFFSET, Palette::new(palette)),
            control: Translation::new(Self::CONTROL_OFFSET, PaletteControl::new()),
            is_collapsed: true,
        }
    }
}
impl <T: Clone> CellPalette<T> {
    pub fn value(&self) -> PaletteResult<T> {
        PaletteResult(
            self.palette.get_current().clone(), 
            self.control.colour, 
            self.control.direction
        )
    }
}
impl<T> Component for CellPalette<T> {
    type Args = ();
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.control.draw(context, assets, ());
        self.palette.draw(context, assets, (self.control.colour, self.control.direction));
    }
    fn bounding_rect(&self) -> Rect {
        self.control
            .combine_dimensions(&self.palette)
            .expand(Point(Self::LEFT_MARGIN, Self::TOP_MARGIN))
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        self.control.click(point) || self.palette.click(point)
    }
}

// no invariants
#[derive(Clone, Debug)]
struct PaletteControl {
    direction: Direction,
    colour: Colour,
}
impl PaletteControl {
    const HEIGHT: i32 = SpriteSheet::STANDARD_HEIGHT;
    const WIDTH: i32 = SpriteSheet::STANDARD_WIDTH * 4;

    const ROTATE_LEFT_GRAPHIC: CellGraphic = CellGraphic {
        offset: Point(0, 0),
        graphic: Point(6, 0),
    };
    const ROTATE_COLOUR_GRAPHIC: CellGraphic = CellGraphic {
        offset: Point(SpriteSheet::STANDARD_WIDTH * 3 / 2, 0),
        graphic: Point(4, 0),
    };
    const ROTATE_RIGHT_GRAPHIC: CellGraphic = CellGraphic {
        offset: Point(SpriteSheet::STANDARD_WIDTH * 3, 0),
        graphic: Point(5, 0),
    };

    fn new() -> Self {
        PaletteControl {
            direction: Direction::default(),
            colour: Colour::default(),
        }
    }
}
impl Component for PaletteControl {
    type Args = ();
    fn bounding_rect(&self) -> Rect {
        Rect {
            top_left: Point(0, 0),
            dimensions: Point(Self::WIDTH, Self::HEIGHT),
        }
    }
    fn click (&mut self, point: Point<i32>) -> bool {
        if Self::ROTATE_LEFT_GRAPHIC.in_boundary(point) {
            self.direction = self.direction.decrement();
            true
        }
        else if Self::ROTATE_COLOUR_GRAPHIC.in_boundary(point) {
            self.colour = self.colour.increment();
            true
        }
        else if Self::ROTATE_RIGHT_GRAPHIC.in_boundary(point) {
            self.direction = self.direction.increment();
            true
        }
        else {
            false
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        Self::ROTATE_LEFT_GRAPHIC.draw(context, assets, ());
        Self::ROTATE_COLOUR_GRAPHIC.draw(context, assets, ());
        Self::ROTATE_RIGHT_GRAPHIC.draw(context, assets, ());
    }
}

// invariant: `current` is a valid index for `entries`.
#[derive(Clone, Debug)]
struct Palette<T> {
    entries: Vec<CellCursorEntry<T>>,
    current: usize,
}
impl<T> Palette<T> {
    const COLUMNS: i32 = 4;
    const CELL_CURSOR_GRAPHIC: Point<u8> = Point(7, 0);

    fn new(entries: Vec<CellCursorEntry<T>>) -> Self {
        Palette {
            entries,
            current: 0,
        }
    }
    fn get_point_from_index(index: usize) -> Point<i32> {
        let index_i32: i32 = index.try_into().unwrap();
        let column = index_i32 % Self::COLUMNS;
        let row = (index_i32 - column) / Self::COLUMNS;
        Point(column, row)
    }
    fn get_index_from_point(&self, point: Point<i32>) -> Option<usize> {
        let Point(x_index, y_index) = point / CellGraphic::CELL_SIZE;
        let absolute_index = (x_index + y_index * Self::COLUMNS).try_into().unwrap();
        if absolute_index >= self.entries.len() {
            None
        }
        else {
            Some(absolute_index)
        }
    }
    fn get_current(&self) -> &T {
        &self.entries[self.current].value
    }
}
impl<T> Component for Palette<T> {
    type Args = (Colour, Direction);
    fn bounding_rect (&self) -> Rect {
        let len_i32: i32 = self.entries.len().try_into().unwrap();
        let rows = (len_i32 + Self::COLUMNS - 1) / Self::COLUMNS;
        let dimensions = Point(Self::COLUMNS, rows) * CellGraphic::CELL_SIZE;
        Rect {
            top_left: Point(0, 0),
            dimensions,
        }
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            false
        }
        else if let Some(index) = self.get_index_from_point(point) {
            self.current = index;
            true
        }
        else {
            false
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, (colour, direction): (Colour, Direction)) {
        for (index, entry) in self.entries.iter().enumerate() {
            let offset = Self::get_point_from_index(index) * CellGraphic::CELL_SIZE;
            let graphic = CellGraphic::new(offset, entry.graphic(colour, direction));

            graphic.draw(context, assets, ());
        }

        let offset = Self::get_point_from_index(self.current) * CellGraphic::CELL_SIZE;
        let cursor_graphic = CellGraphic::new(offset, Self::CELL_CURSOR_GRAPHIC);
        cursor_graphic.draw(context, assets, ());
    }
}

#[derive(Clone, Debug)]
pub struct CellCursorEntry<T> {
    value: T,
    // base graphic for default colour and direction.
    graphic: Point<u8>,
    has_colour: bool,
    has_direction: bool,
}
impl From<CellType> for CellCursorEntry<CellType> {
    fn from (cell_type: CellType) -> Self {
        match cell_type {
            CellType::Empty => Self::new(cell_type, Point(8, 0), false, false),
            CellType::ColouredBlock => Self::new(cell_type, Point(0, 0), true, false),
            CellType::Arrow => Self::new(cell_type, Point(0, 7), false, true),
            CellType::ColouredArrow => Self::new(cell_type, Point(0, 3), true, true),
            CellType::ArrowBlock => Self::new(cell_type, Point(0, 1), false, true),
            CellType::RotateRight => Self::new(cell_type, Point(0, 2), false, false),
            CellType::RotateLeft => Self::new(cell_type, Point(1, 2), false, false),
            CellType::Fence => Self::new(cell_type, Point(0, 14), false, false),
            CellType::Wall => Self::new(cell_type, Point(0, 15), false, false),
            CellType::Overlay => Self::new(cell_type, Point(9, 0), true, false),
        }
    }
}
impl<T> CellCursorEntry<T> {
    fn new(value: T, graphic: Point<u8>, has_colour: bool, has_direction: bool) -> Self {
        CellCursorEntry {
            value,
            graphic,
            has_colour,
            has_direction,
        }
    }
    fn graphic(&self, colour: Colour, direction: Direction) -> Point<u8> {
        let mut sprite_index_offset = 0;
        if self.has_direction {
            sprite_index_offset += direction as u8;
            if self.has_colour {
                sprite_index_offset += (colour as u8) * Direction::TOTAL_DIRECTIONS;
            }
        }
        else if self.has_colour { // and not direction
            sprite_index_offset += colour as u8;
        }

        Point(self.graphic.x() + sprite_index_offset, self.graphic.y())
    }
}

#[derive(Clone, Debug)]
struct CellGraphic {
    offset: Point<i32>,
    graphic: Point<u8>,
}
impl CellGraphic {
    const CELL_SIZE: Point<i32> = Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
    fn new(offset: Point<i32>, graphic: Point<u8>) -> Self {
        CellGraphic {
            offset,
            graphic,
        }
    }
}
impl Component for CellGraphic {
    type Args = ();
    fn bounding_rect(&self) -> Rect {
        Rect {
            top_left: self.offset,
            dimensions: Self::CELL_SIZE,
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let point = Point(self.offset.x().into(), self.offset.y().into());
        assets.blocks.draw(context, self.graphic, point);
    }
}

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
        match cell {
            OverlayCell::Empty => CellCursor(CellType::Empty, Colour::Red, Direction::Up),
            OverlayCell::Success(_) => CellCursor(CellType::Overlay, Colour::Green, Direction::Up),
            OverlayCell::Failure(_) => CellCursor(CellType::Overlay, Colour::Red, Direction::Up),
            OverlayCell::Input(_) => CellCursor(CellType::Overlay, Colour::Orange, Direction::Up),
            OverlayCell::Output(_) => CellCursor(CellType::Overlay, Colour::Blue, Direction::Up),
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
