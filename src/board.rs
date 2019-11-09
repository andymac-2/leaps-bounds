use im_rc::OrdMap;
use serde::{Deserialize, Serialize};

use crate::cell::{Cell, CellCursor};
use crate::js_ffi::draw_layer;
use crate::{console_log, Context2D, Point, SpriteSheet};

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

    pub fn set_cell(&mut self, point: Point<i32>, cell: Cell) {
        if cell == self.default_cell {
            let _ = self.board.remove(&point);
        } else {
            self.board.insert(point, cell);
        }
    }

    pub fn left_click(&mut self, point: Point<i32>, cursor: CellCursor) {
        let x_index = point.x() / (SpriteSheet::STANDARD_WIDTH as i32);
        let y_index = point.y() / (SpriteSheet::STANDARD_HEIGHT as i32);
        let index = Point(x_index, y_index);

        self.set_cell(index, cursor.into());
    }

    pub fn log_board(&self) {
        console_log!("{}", serde_json::to_string(self).unwrap());
    }

    pub fn draw(
        &self,
        context: &Context2D,
        blocks: &SpriteSheet,
        top_left: Point<i32>,
        dimensions: Point<i32>,
    ) {
        assert!(dimensions.x() >= 0);
        assert!(dimensions.y() >= 0);
        let left = top_left.x();
        let right = dimensions.x() + left;
        let top = top_left.y();
        let bottom = dimensions.y() + top;

        let mut buffer = Vec::with_capacity((dimensions.x() * dimensions.y() * 2) as usize);

        for y_index in top..bottom {
            for x_index in left..right {
                self.get_cell(&Point(x_index, y_index))
                    .draw_into_buffer(&mut buffer);
            }
        }

        draw_layer(
            context,
            blocks.get_image(),
            SpriteSheet::STANDARD_WIDTH,
            SpriteSheet::STANDARD_HEIGHT,
            buffer.as_ptr(),
            dimensions.x(),
            dimensions.y(),
        );
    }
}
