use im_rc::OrdMap;
use serde::{Deserialize, Serialize};

use crate::cell::{Cell, CellCursor, Direction};
use crate::js_ffi::draw_layer;
use crate::{Context2D, Image, Point, SpriteSheet};

// #[derive(Debug, Clone)]
// pub struct Serial<T>(T);
// impl<T> std::ops::Deref for Serial<T> {
//     type Target = T;
//     fn deref (&self) -> &T {
//         &self.0
//     }
// }
// impl<T> std::ops::DerefMut for Serial<T> {
//     fn deref_mut (&mut self) -> &mut T {
//         &mut self.0
//     }
// }
// impl<K, V> Serialize for Serial<OrdMap<K, V>>
// where
//     K: Serialize + Ord,
//     V: Serialize,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let entries: Vec<_> = self.0.iter().collect();
//         serializer.serialize_newtype_struct("Serial", &entries)
//     }
// }
// impl<'de, K, V> Deserialize<'de> for Serial<OrdMap<K, V>>
// where
//     K: Deserialize<'de> + Clone + Ord,
//     V: Deserialize<'de> + Clone,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let entries = <Vec<(K, V)>>::deserialize(deserializer)?;
//         Ok(Serial(entries.into()))
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Board {
    board: OrdMap<Point<i32>, Cell>,
    default_cell: Cell,
}
impl Board {
    pub fn new(default_cell: Cell) -> Self {
        Board {
            board: OrdMap::new(),
            default_cell,
        }
    }

    pub fn get_cell(&self, point: &Point<i32>) -> &Cell {
        self.board.get(point).unwrap_or(&self.default_cell)
    }

    pub fn set_cell(&mut self, point: Point<i32>, mut cell: Cell) {
        let mut set_surrounds = |direction| {
            let mut adjacent = point.clone();
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

    // does not check for surrounding tiles when inserting.
    fn set_cell_unchecked(&mut self, point: Point<i32>, cell: Cell) {
        if cell == self.default_cell {
            let _ = self.board.remove(&point);
        } else {
            self.board.insert(point, cell);
        }
    }

    // does not check for surrounding tiles when inserting.
    fn map_cell_unchecked<F>(&mut self, point: Point<i32>, func: F)
    where
        F: FnOnce(Cell) -> Cell,
    {
        self.set_cell_unchecked(point, func(*self.get_cell(&point)));
    }

    pub fn map_cell<F>(&mut self, point: Point<i32>, func: F)
    where
        F: FnOnce(Cell) -> Cell,
    {
        self.set_cell(point, func(*self.get_cell(&point)));
    }

    pub fn left_click(&mut self, point: Point<i32>, cursor: CellCursor) {
        let x_index = point.x() / (SpriteSheet::STANDARD_WIDTH as i32);
        let y_index = point.y() / (SpriteSheet::STANDARD_HEIGHT as i32);
        let index = Point(x_index, y_index);

        self.set_cell(index, cursor.into());
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
