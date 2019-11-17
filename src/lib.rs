use wasm_bindgen::prelude::*;

mod component;
mod scene;
mod direction;
mod js_ffi;
mod level;
mod point;
mod sprite_sheet;
mod state_stack;
mod util;

use js_ffi::KeyboardState;
use point::Point;
use sprite_sheet::SpriteSheet;
use scene::{Scene, Scenes};
use component::Component;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

type Context2D = web_sys::CanvasRenderingContext2d;
type Image = web_sys::HtmlImageElement;

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (crate::js_ffi::log(&format_args!($($t)*).to_string()))
}
#[macro_export]
macro_rules! here {
    () => (crate::console_log!("Arrived at {} line {}.", file!(), line!()))
}

#[wasm_bindgen]
pub struct Assets {
    blocks: SpriteSheet,
    sprites: SpriteSheet,
}
#[wasm_bindgen]
impl Assets {
    pub fn new(blocks: Image, sprites: Image) -> Self {
        Assets {
            blocks: SpriteSheet::default_size_new(blocks),
            sprites: SpriteSheet::default_size_new(sprites),
        }
    }
}

#[wasm_bindgen]
pub struct LeapsAndBounds {
    scenes: Scenes,
    keyboard_state: KeyboardState,
}
impl Default for LeapsAndBounds {
    fn default() -> Self {
        Self::new()
    }
}
#[wasm_bindgen]
impl LeapsAndBounds {
    pub fn new() -> Self {
        // This provides better error messages in debug mode.
        // It's disabled in release mode so it doesn't bloat up the file size.
        #[cfg(debug_assertions)]
        console_error_panic_hook::set_once();

        LeapsAndBounds {
            scenes: Scenes::new(),
            keyboard_state: KeyboardState::new(),
        }
    }
    pub fn step(&mut self, dt: f64) {
        self.scenes.step(dt, &self.keyboard_state);
        self.keyboard_state.tick();
    }
    pub fn draw(&self, context: &Context2D, assets: &Assets) {
        context.save();
        self.scenes.draw(context, assets, ());
        context.restore();
    }
    pub fn left_click(&mut self, x: i32, y: i32) {
        self.scenes.click(Point(x, y));
    }
}
