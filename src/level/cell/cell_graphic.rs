use crate::{SpriteSheet, component, Context2D, Assets};

use crate::point::Point;

#[derive(Clone, Debug)]
pub struct CellGraphic {
    offset: Point<i32>,
    graphic: Point<u8>,
}
impl CellGraphic {
    pub const CELL_SIZE: Point<i32> = Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
    pub const fn new(offset: Point<i32>, graphic: Point<u8>) -> Self {
        CellGraphic { offset, graphic }
    }
    pub fn set_graphic(&mut self, point: Point<u8>) {
        self.graphic = point;
    }
    pub fn set_offset(&mut self, point: Point<i32>) {
        self.offset = point;
    }
}
impl component::Component for CellGraphic {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        component::Rect {
            top_left: self.offset,
            dimensions: Self::CELL_SIZE,
        }
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        self.in_boundary(point)
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let point = Point(self.offset.x().into(), self.offset.y().into());
        assets.blocks.draw(context, self.graphic, point);
    }
}