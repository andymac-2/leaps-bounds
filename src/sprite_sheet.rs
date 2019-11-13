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
}
