use crate::{component, Assets, Context2D, SpriteSheet};

use crate::point::Point;

#[derive(Clone, Debug)]
pub struct CellGraphic {
    offset: Point<i32>,
    graphic: Point<u8>,
}
impl CellGraphic {
    pub const CELL_SIZE: Point<i32> =
        Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
    pub const fn new(offset: Point<i32>, graphic: Point<u8>) -> Self {
        CellGraphic { offset, graphic }
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
