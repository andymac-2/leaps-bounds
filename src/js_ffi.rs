use wasm_bindgen::prelude::*;

use crate::{Context2D, Image};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    pub fn error(s: &str);

    pub type BasicAudioPlayer;
    #[wasm_bindgen(constructor)]
    pub fn new() -> BasicAudioPlayer;
    #[wasm_bindgen(method)]
    pub fn play_sound(this: &BasicAudioPlayer, id: &str);

    pub type KeyboardState;
    #[wasm_bindgen(constructor)]
    pub fn new() -> KeyboardState;
    #[wasm_bindgen(method)]
    pub fn is_pressed(this: &KeyboardState, code: &str) -> bool;
    #[wasm_bindgen(method)]
    pub fn is_held(this: &KeyboardState, code: &str) -> bool;
    #[wasm_bindgen(method)]
    pub fn tick(this: &KeyboardState);

    #[wasm_bindgen]
    pub fn draw_layer(
        context: &Context2D,
        image: &Image,
        sprite_width: i32,
        sprite_height: i32,
        data: *const u8,
        width: i32,
        height: i32,
    );
    #[wasm_bindgen]
    pub fn draw_rope(context: &Context2D, start_x: f64, start_y: f64, end_x: f64, end_y: f64);
}
