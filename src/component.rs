use std::collections::HashMap;

mod transition;

pub use transition::Transition;

use crate::point::Point;
use crate::util::with_saved_context;
use crate::{Assets, Context2D, KeyboardState, SpriteSheet};

pub trait Component {
    type DrawArgs;

    fn bounding_rect(&self) -> Rect;
    fn step(&mut self, _dt: f64, _keyboard_state: &KeyboardState) -> NextScene {
        NextScene::Continue
    }
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::DrawArgs);

    // performs a click event on a given component. returns true if the event
    // was handled.
    fn click(&mut self, _point: Point<i32>) -> bool {
        false
    }
    /// Default behaviour assumes an AABB
    fn in_boundary(&self, point: Point<i32>) -> bool {
        let Rect {
            top_left,
            dimensions,
        } = self.bounding_rect();
        let local_point = point - top_left;

        local_point.x() >= 0
            && local_point.x() < dimensions.x()
            && local_point.y() >= 0
            && local_point.y() < dimensions.y()
    }
    fn top_left(&self) -> Point<i32> {
        self.bounding_rect().top_left
    }
    fn dimensions(&self) -> Point<i32> {
        self.bounding_rect().dimensions
    }
    fn draw_bbox(&self, context: &Context2D, colour: &str) {
        let rect = self.bounding_rect();

        context.set_stroke_style(&wasm_bindgen::JsValue::from_str(colour));
        context.stroke_rect(
            rect.top_left.x().into(),
            rect.top_left.y().into(),
            rect.dimensions.x().into(),
            rect.dimensions.y().into(),
        );
    }
    fn fill_bg(&self, context: &Context2D, colour: &str) {
        let rect = self.bounding_rect();

        context.set_fill_style(&wasm_bindgen::JsValue::from_str(colour));
        context.fill_rect(
            rect.top_left.x().into(),
            rect.top_left.y().into(),
            rect.dimensions.x().into(),
            rect.dimensions.y().into(),
        );
    }

    fn returned_into(&mut self, _object: Object) {}
    fn called_into(&mut self, _object: Object) {}
    fn jumped_into(&mut self, object: Object) {
        self.called_into(object)
    }
}

// A generic data object, kind of like JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<Object>),
    Map(HashMap<String, Object>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NextScene {
    Continue,
    Return(Object),
    Call(usize, Object),
    Jump(usize, Object),
}

// invariant: dimensions are positive
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub top_left: Point<i32>,
    pub dimensions: Point<i32>,
}
impl Rect {
    pub const fn new(top_left: Point<i32>, dimensions: Point<i32>) -> Rect {
        Rect {
            top_left,
            dimensions,
        }
    }
    pub fn expand(&self, increase: Point<i32>) -> Rect {
        Rect {
            top_left: self.top_left - increase,
            dimensions: self.dimensions + increase + increase,
        }
    }
    pub fn combine(&self, other: &Rect) -> Rect {
        let self_bot_right = self.top_left + self.dimensions;
        let other_bot_right = other.top_left + other.dimensions;

        let tl_x = self.top_left.x().min(other.top_left.x());
        let tl_y = self.top_left.y().min(other.top_left.y());

        let br_x = self_bot_right.x().max(other_bot_right.x());
        let br_y = self_bot_right.y().max(other_bot_right.y());

        Rect {
            top_left: Point(tl_x, tl_y),
            dimensions: Point(br_x - tl_x, br_y - tl_y),
        }
    }
    pub fn translate(&self, translation: Point<i32>) -> Rect {
        let top_left = self.top_left + translation;
        Rect::new(top_left, self.dimensions)
    }

    pub const fn indexed(index: Point<u8>, dimensions: Point<i32>) -> Rect {
        Rect::new(
            Point(index.0 as i32 * dimensions.0, index.1 as i32 * dimensions.1),
            dimensions,
        )
    }

    pub const ONE_BY_ONE: Point<i32> =
        Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
    pub const TWO_BY_TWO: Point<i32> = Point(
        SpriteSheet::STANDARD_WIDTH * 2,
        SpriteSheet::STANDARD_HEIGHT * 2,
    );
    pub const FOUR_BY_FOUR: Point<i32> = Point(
        SpriteSheet::STANDARD_WIDTH * 4,
        SpriteSheet::STANDARD_HEIGHT * 4,
    );
    pub const FOUR_BY_TWO: Point<i32> = Point(
        SpriteSheet::STANDARD_WIDTH * 4,
        SpriteSheet::STANDARD_HEIGHT * 2,
    );
}

pub fn combine_dimensions<A: Component, B: Component>(one: &A, other: &B) -> Rect {
    one.bounding_rect().combine(&other.bounding_rect())
}

#[derive(Clone, Debug)]
pub struct Translation<T> {
    pub translation: Point<i32>,
    pub component: T,
}
impl<T> Translation<T> {
    pub fn new(translation: Point<i32>, component: T) -> Self {
        Translation {
            translation,
            component,
        }
    }
    fn get_local_point(&self, point: Point<i32>) -> Point<i32> {
        point - self.translation
    }
}
impl<T> std::ops::Deref for Translation<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}
impl<T> std::ops::DerefMut for Translation<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
impl<T: Component> Component for Translation<T> {
    type DrawArgs = T::DrawArgs;
    fn bounding_rect(&self) -> Rect {
        self.component.bounding_rect().translate(self.translation)
    }
    fn step(&mut self, dt: f64,  keyboard_state: &KeyboardState) -> NextScene {
        self.component.step(dt, keyboard_state)
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !Component::in_boundary(self, point) {
            return false;
        }
        let local_point = self.get_local_point(point);

        self.component.click(local_point)
    }
    fn in_boundary(&self, point: Point<i32>) -> bool {
        let local_point = point - self.translation;
        self.component.in_boundary(local_point)
    }
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::DrawArgs) {
        with_saved_context(context, || {
            context
                .translate(self.translation.x().into(), self.translation.y().into())
                .unwrap();
            self.component.draw(context, assets, args);
        });
    }
    fn returned_into(&mut self, object: Object) {
        self.component.returned_into(object)
    }
    fn called_into(&mut self, object: Object) {
        self.component.called_into(object)
    }
    fn jumped_into(&mut self, object: Object) {
        self.component.jumped_into(object)
    }
}
