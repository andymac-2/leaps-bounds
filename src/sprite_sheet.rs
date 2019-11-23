use std::convert::TryInto;

use crate::component::Rect;
use crate::point::Point;
use crate::{Context2D, Image};


#[derive(Debug, Clone)]
pub struct SpriteSheet {
    image: Image,
    sprite_width: i32,
    sprite_height: i32,
}

impl SpriteSheet {
    pub const STANDARD_WIDTH: i32 = 16;
    pub const STANDARD_HEIGHT: i32 = 16;

    pub fn new(image: Image, sprite_width: i32, sprite_height: i32) -> Self {
        SpriteSheet {
            image,
            sprite_height,
            sprite_width,
        }
    }
    pub fn default_size_new(image: Image) -> Self {
        SpriteSheet::new(
            image,
            SpriteSheet::STANDARD_HEIGHT,
            SpriteSheet::STANDARD_HEIGHT,
        )
    }
    pub fn get_image(&self) -> &Image {
        &self.image
    }
    pub fn draw(&self, context: &Context2D, sprite_index: Point<u8>, offset: Point<f64>) {
        let sx = f64::from(sprite_index.x()) * f64::from(self.sprite_width);
        let sy = f64::from(sprite_index.y()) * f64::from(self.sprite_height);
        let width = f64::from(self.sprite_width);
        let height = f64::from(self.sprite_height);

        context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.image,
                sx,
                sy,
                width,
                height,
                offset.x(),
                offset.y(),
                width,
                height,
            )
            .unwrap();
    }
    pub fn draw_with_rect(&self, context: &Context2D, source: &Rect, dest: &Rect) {
        context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.image,
                source.top_left.x().into(),
                source.top_left.y().into(),
                source.dimensions.x().into(),
                source.dimensions.y().into(),
                dest.top_left.x().into(),
                dest.top_left.y().into(),
                dest.dimensions.x().into(),
                dest.dimensions.y().into(),
            )
            .unwrap();
    }
    pub fn draw_with_source_height(&self, context: &Context2D, source: &Rect, dest_centre: Point<i32>, dest_height: i32) {
        let dest_width = source.dimensions.x() * dest_height / source.dimensions.y();
        let dest_left = dest_centre.x() - (dest_width / 2);
        let dest_top = dest_centre.y() - (dest_height / 2);

        context
            .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &self.image,
                source.top_left.x().into(),
                source.top_left.y().into(),
                source.dimensions.x().into(),
                source.dimensions.y().into(),
                dest_left.into(),
                dest_top.into(),
                dest_width.into(),
                dest_height.into(),
            )
            .unwrap();
    }
}
