use wasm_bindgen::prelude::*;

mod component;
mod direction;
mod js_ffi;
mod level;
mod point;
mod scene;
mod sprite_sheet;
mod state_stack;
mod tutorial;
mod util;

use component::Component;
use js_ffi::{KeyboardState, BasicAudioPlayer};
use point::Point;
use scene::Scenes;
use sprite_sheet::SpriteSheet;

const DEBUG: bool = true;

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
macro_rules! console_error {
    ($($t:tt)*) => {{
        let user_string = std::fmt::format(format_args!($($t)*));
        let string = std::fmt::format(format_args!(
            "Error at {} line {}: {}", file!(), line!(), user_string)); 
        crate::js_ffi::error(&string)
    }}
}
#[macro_export]
macro_rules! here {
    () => {
        crate::console_log!("Arrived at {} line {}.", file!(), line!())
    };
}

#[wasm_bindgen]
pub struct Assets {
    blocks: SpriteSheet,
    sprites: SpriteSheet,
    misc: SpriteSheet,
}
#[wasm_bindgen]
impl Assets {
    pub fn new(blocks: Image, sprites: Image, misc: Image) -> Self {
        Assets {
            blocks: SpriteSheet::default_size_new(blocks),
            sprites: SpriteSheet::default_size_new(sprites),
            misc: SpriteSheet::default_size_new(misc),
        }
    }
}

#[wasm_bindgen]
pub struct LeapsAndBounds {
    scenes: Scenes,
    keyboard_state: KeyboardState,
    audio: js_ffi::BasicAudioPlayer,
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
            audio: BasicAudioPlayer::new()
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
        self.audio.play_sound("thinking");
        self.scenes.click(Point(x, y));
    }
}
