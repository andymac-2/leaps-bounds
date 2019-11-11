use std::convert::TryFrom;

use im_rc::OrdMap;
use serde::{Deserialize, Serialize};

use super::cell::{Cell, CellCursor, GroundCell, OverlayCell};
use crate::direction::Direction;
use crate::js_ffi::draw_layer;
use crate::{Context2D, Image, Point, SpriteSheet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelLayer<T: Clone> {
    layer: OrdMap<Point<i32>, T>,
    default: T,
}
impl<T> LevelLayer<T>
where
    T: Clone + PartialEq + Cell,
{
    pub fn new(default: T) -> Self {
        LevelLayer {
            layer: OrdMap::new(),
            default,
        }
    }
    pub fn get_cell(&self, point: &Point<i32>) -> &T {
        self.layer.get(point).unwrap_or(&self.default)
    }

    pub fn set_cell(&mut self, point: Point<i32>, mut cell: T) {
        let mut set_surrounds = |direction| {
            let mut adjacent = point;
            adjacent.increment_2d(direction);

            self.map_cell_unchecked(adjacent, |mut other| {
                cell.calculate_surround(&mut other, direction);
                other
            })
        };

        set_surrounds(Direction::Up);
        set_surrounds(Direction::Right);
        set_surrounds(Direction::Down);
        set_surrounds(Direction::Left);

        self.set_cell_unchecked(point, cell);
    }

    pub fn map_cell<F>(&mut self, point: Point<i32>, func: F)
    where
        F: FnOnce(T) -> T,
    {
        self.set_cell(point, func(self.get_cell(&point).clone()));
    }

    fn set_cell_unchecked(&mut self, point: Point<i32>, cell: T) {
        if cell == self.default {
            let _ = self.layer.remove(&point);
        } else {
            self.layer.insert(point, cell);
        }
    }

    fn map_cell_unchecked<F>(&mut self, point: Point<i32>, func: F)
    where
        F: FnOnce(T) -> T,
    {
        self.set_cell_unchecked(point, func(self.get_cell(&point).clone()));
    }

    pub fn draw(
        &self,
        context: &Context2D,
        blocks: &SpriteSheet,
        top_left: Point<i32>,
        dimensions: Point<i32>,
    ) {
        let cell_size = Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
        let mut layer = Layer::new(top_left, dimensions, cell_size, cell_size);

        assert!(dimensions.x() >= 0);
        assert!(dimensions.y() >= 0);

        for y_index in 0..dimensions.y() {
            for x_index in 0..dimensions.x() {
                self.get_cell(&Point(x_index, y_index))
                    .draw_into_layer(&mut layer);
            }
        }

        layer.draw(context, blocks.get_image());
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    ground: LevelLayer<GroundCell>,
    overlay: LevelLayer<OverlayCell>,
}
impl Board {
    pub fn new(default_cell: GroundCell, default_overlay: OverlayCell) -> Self {
        Board {
            ground: LevelLayer::new(default_cell),
            overlay: LevelLayer::new(default_overlay),
        }
    }

    pub fn get_ground_cell(&self, point: &Point<i32>) -> &GroundCell {
        self.ground.get_cell(point)
    }
    pub fn get_overlay_cell(&self, point: &Point<i32>) -> &OverlayCell {
        self.overlay.get_cell(point)
    }
    pub fn set_ground_cell(&mut self, point: Point<i32>, cell: GroundCell) {
        self.ground.set_cell(point, cell);
    }
    pub fn map_ground_cell<F>(&mut self, point: Point<i32>, func: F)
    where
        F: FnOnce(GroundCell) -> GroundCell,
    {
        self.ground.map_cell(point, func)
    }

    pub fn left_click(&mut self, point: Point<i32>, cursor: CellCursor) {
        let x_index = point.x() / (SpriteSheet::STANDARD_WIDTH as i32);
        let y_index = point.y() / (SpriteSheet::STANDARD_HEIGHT as i32);
        let index = Point(x_index, y_index);

        if let Ok(cell) = GroundCell::try_from(cursor) {
            self.ground.set_cell(index, cell)
        } else if let Ok(cell) = OverlayCell::try_from(cursor) {
            self.overlay.set_cell(index, cell)
        }
    }

    pub fn draw_ground(
        &self,
        context: &Context2D,
        blocks: &SpriteSheet,
        top_left: Point<i32>,
        dimensions: Point<i32>,
    ) {
        self.ground.draw(context, blocks, top_left, dimensions);
    }

    pub fn draw_overlay(
        &self,
        context: &Context2D,
        blocks: &SpriteSheet,
        top_left: Point<i32>,
        dimensions: Point<i32>,
    ) {
        self.overlay.draw(context, blocks, top_left, dimensions);
    }
}

pub struct Layer {
    top_left: Point<i32>,
    grid_dimensions: Point<i32>,
    cell_dimensions: Point<u32>,
    dest_cell_dimensions: Point<u32>,
    buffer: Vec<u8>,
}
impl Layer {
    const EMPTY: u8 = 255;
    /// top left in pixels, prid dimensions in number of cells wide, cell
    /// dimensions in pixels, destination cell dimensions in pixels
    pub fn new(
        top_left: Point<i32>,
        grid_dimensions: Point<i32>,
        cell_dimensions: Point<u32>,
        dest_cell_dimensions: Point<u32>,
    ) -> Self {
        assert!(grid_dimensions.x() >= 0);
        assert!(grid_dimensions.y() >= 0);
        let capacity = (grid_dimensions.x() * grid_dimensions.y() * 2) as usize;

        Layer {
            top_left,
            grid_dimensions,
            cell_dimensions,
            dest_cell_dimensions,
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn add_cell(&mut self, sprite_offset: Option<Point<u8>>) {
        let Point(x, y) = sprite_offset.unwrap_or(Point(Layer::EMPTY, Layer::EMPTY));
        self.buffer.push(x);
        self.buffer.push(y);
    }
    pub fn draw(&self, context: &Context2D, image: &Image) {
        assert!(
            self.buffer.len() == (self.grid_dimensions.x() * self.grid_dimensions.y() * 2) as usize
        );
        draw_layer(
            context,
            image,
            self.cell_dimensions.x(),
            self.cell_dimensions.y(),
            self.buffer.as_ptr(),
            self.grid_dimensions.x(),
            self.grid_dimensions.y(),
        );
    }
}
