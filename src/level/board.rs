use std::convert::{TryFrom, TryInto};

use im_rc::OrdMap;
use serde::{Deserialize, Serialize};

use super::cell::{Cell, CellType, Colour, GroundCell, OverlayCell, PaletteResult};
use super::NotEnoughInputSpace;
use crate::direction::Direction;
use crate::js_ffi::draw_layer;
use crate::{Context2D, Image, Point, SpriteSheet};

pub fn get_grid_index(point: Point<i32>) -> Point<i32> {
    let x_index = point.x() / (SpriteSheet::STANDARD_WIDTH as i32);
    let y_index = point.y() / (SpriteSheet::STANDARD_HEIGHT as i32);
    Point(x_index, y_index)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelLayer<T: Clone> {
    layer: OrdMap<Point<i32>, T>,
    default: T,
}
impl<T> Default for LevelLayer<T>
where
    T: Clone + PartialEq + Cell + Default,
{
    fn default() -> Self {
        LevelLayer {
            layer: OrdMap::new(),
            default: T::default(),
        }
    }
}
impl<T> LevelLayer<T>
where
    T: Clone + PartialEq + Cell,
{
    const CELL_SIZE: Point<i32> = Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
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
        Direction::for_every(|direction| {
            let mut adjacent = point;
            adjacent.increment_2d(direction);

            self.map_cell_unchecked(adjacent, |mut other| {
                cell.calculate_surround(&mut other, direction);
                other
            })
        });

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
        let mut layer = Layer::new(top_left, dimensions, Self::CELL_SIZE, Self::CELL_SIZE);

        assert!(dimensions.x() >= 0);
        assert!(dimensions.y() >= 0);

        for (point, cell) in self.layer.iter() {
            if !point.is_inside(dimensions) {
                continue;
            }

            while layer.cursor() < *point {
                self.default.draw_into_layer(&mut layer);
            }

            cell.draw_into_layer(&mut layer);
        }

        while !layer.is_full() {
            self.default.draw_into_layer(&mut layer);
        }

        layer.draw(context, blocks.get_image());
    }
}
impl<T> super::Pasture<T> for LevelLayer<T>
where
    T: Clone + PartialEq + Cell,
{
    fn get_pasture_cell(&self, point: Point<i32>) -> &T {
        self.get_cell(&point)
    }
}
impl LevelLayer<OverlayCell> {
    pub fn get_input_coordinates(&self) -> Vec<Point<i32>> {
        self.layer
            .iter()
            .filter_map(|(point, overlay_cell)| {
                if let OverlayCell::Input(_) = overlay_cell {
                    Some(*point)
                } else {
                    None
                }
            })
            .collect()
    }
}
impl LevelLayer<OverlayCell> {
    pub fn get_output_coordinates(&self) -> Vec<Point<i32>> {
        self.layer
            .iter()
            .filter_map(|(point, overlay_cell)| {
                if let OverlayCell::Output(_) = overlay_cell {
                    Some(*point)
                } else {
                    None
                }
            })
            .collect()
    }
}
impl LevelLayer<GroundCell> {
    pub fn get_coloured_blocks(&self, coordinates: &Vec<Point<i32>>) -> Vec<Colour> {
        coordinates
            .iter()
            .filter_map(|point| {
                if let GroundCell::ColouredBlock(colour) = self.get_cell(point) {
                    Some(*colour)
                } else {
                    None
                }
            })
            .collect()
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
    pub fn get_outputs(&self) -> Vec<Colour> {
        let output_coordinates = self.overlay.get_output_coordinates();
        self.ground.get_coloured_blocks(&output_coordinates)
    }
    pub fn get_inputs(&self) -> Vec<Colour> {
        let input_coordinates = self.overlay.get_input_coordinates();
        self.ground.get_coloured_blocks(&input_coordinates)
    }

    /// Sets the input overlay area as coloured blocks. Returns false and leaves
    /// the board unchanged if the input area is loess than the input size. It
    /// will return true if the input fits inside of the input area.
    pub fn set_inputs(&mut self, input: &Vec<Colour>) -> Result<(), NotEnoughInputSpace> {
        let input_coordinates = self.overlay.get_input_coordinates();
        if input_coordinates.len() < input.len() {
            return Err(NotEnoughInputSpace);
        };

        let mut input_iter = input.iter();
        for coordinate in input_coordinates.iter() {
            if let Some(colour) = input_iter.next() {
                self.set_ground_cell(*coordinate, GroundCell::ColouredBlock(*colour));
            } else {
                self.set_ground_cell(*coordinate, GroundCell::Empty);
            }
        }

        Ok(())
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

    pub fn set_cell_at_point(&mut self, point: Point<i32>, cell_type: PaletteResult<CellType>) {
        let index = get_grid_index(point);

        if let Ok(cell) = GroundCell::try_from(cell_type) {
            self.ground.set_cell(index, cell)
        }
        if let Ok(cell) = OverlayCell::try_from(cell_type) {
            self.overlay.set_cell(index, cell)
        }
    }

    pub fn get_overlay_cell_at_point(&mut self, point: Point<i32>) -> OverlayCell {
        let index = get_grid_index(point);
        *self.get_overlay_cell(&index)
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
impl super::Pasture<GroundCell> for Board {
    fn get_pasture_cell(&self, point: Point<i32>) -> &GroundCell {
        self.get_ground_cell(&point)
    }
}

pub struct Layer {
    top_left: Point<i32>,
    grid_dimensions: Point<i32>,
    cell_dimensions: Point<i32>,
    dest_cell_dimensions: Point<i32>,
    buffer: Vec<u8>,
}
impl Layer {
    const EMPTY: u8 = 255;
    /// top left in pixels, prid dimensions in number of cells wide, cell
    /// dimensions in pixels, destination cell dimensions in pixels
    pub fn new(
        top_left: Point<i32>,
        grid_dimensions: Point<i32>,
        cell_dimensions: Point<i32>,
        dest_cell_dimensions: Point<i32>,
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
        assert!(self.buffer.len() % 2 == 0);
        assert!(
            self.buffer.len() < (self.grid_dimensions.x() * self.grid_dimensions.y() * 2) as usize
        );

        let Point(x, y) = sprite_offset.unwrap_or(Point(Layer::EMPTY, Layer::EMPTY));
        self.buffer.push(x);
        self.buffer.push(y);
    }
    pub fn cursor(&self) -> Point<i32> {
        assert!(self.buffer.len() % 2 == 0);
        assert!(
            self.buffer.len() <= (self.grid_dimensions.x() * self.grid_dimensions.y() * 2) as usize
        );
        let length: i32 = (self.buffer.len() / 2).try_into().unwrap();

        let x = length % self.grid_dimensions.x();
        let y = (length - x) / self.grid_dimensions.x();
        Point(x, y)
    }
    pub fn is_full(&self) -> bool {
        assert!(
            self.buffer.len() <= (self.grid_dimensions.x() * self.grid_dimensions.y() * 2) as usize
        );
        self.buffer.len()
            == (self.grid_dimensions.x() * self.grid_dimensions.y() * 2)
                .try_into()
                .unwrap()
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
