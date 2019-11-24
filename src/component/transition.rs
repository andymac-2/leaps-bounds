use crate::{Context2D, Assets, KeyboardState, util, point};

use super::NextScene;

pub struct Transition<T> {
    scene: T,
    state: TransitionState,
}
pub enum TransitionState {
    In(f64),
    Running,
    Out(NextScene, f64),
}
impl<T> Transition<T> {
    pub fn new(scene: T) -> Self {
        Transition {
            scene,
            state: TransitionState::In(0.0),
        }
    }
    const TOTAL_TIME: f64 = 400.0;
    const SCREEN_DIMS: super::Rect = crate::level::cow_level::CowLevel::BOUNDING_RECT;
    fn draw_box_in(context: &Context2D, mut animation_time: f64) {
        animation_time = Self::TOTAL_TIME - animation_time;
        let anim_progress = util::clamp(animation_time, 0.0, Self::TOTAL_TIME) / Self::TOTAL_TIME;

        let width = f64::from(Self::SCREEN_DIMS.dimensions.x()) * anim_progress;
        let height = f64::from(Self::SCREEN_DIMS.dimensions.y()) * anim_progress;

        let top = f64::from(Self::SCREEN_DIMS.dimensions.y()) - height;
        let left = f64::from(Self::SCREEN_DIMS.dimensions.x()) - width;

        context.set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
        context.fill_rect(left, top, width, height);
    }
    fn draw_box_out(context: &Context2D, animation_time: f64) {
        let anim_progress = util::clamp(animation_time, 0.0, Self::TOTAL_TIME) / Self::TOTAL_TIME;

        let width = f64::from(Self::SCREEN_DIMS.dimensions.x()) * anim_progress;
        let height = f64::from(Self::SCREEN_DIMS.dimensions.y()) * anim_progress;

        context.set_fill_style(&wasm_bindgen::JsValue::from_str("black"));
        context.fill_rect(0.0, 0.0, width, height);
    }

    fn reset(&mut self) {
        self.state = TransitionState::In(0.0);
    }
}

impl<T> super::Component for Transition<T>
where 
    T: super::Component,
{
    type DrawArgs = T::DrawArgs;
    fn bounding_rect(&self) -> super::Rect {
        self.scene.bounding_rect()
    }
    fn step(&mut self, dt: f64, keyboard: &KeyboardState) -> super::NextScene {
        match &mut self.state {
            TransitionState::In(animation_time) => {
                *animation_time += dt;
                if *animation_time > Self::TOTAL_TIME {
                    self.state = TransitionState::Running;
                };
                NextScene::Continue
            },
            TransitionState::Running => {
                let result = self.scene.step(dt, keyboard);
                if NextScene::Continue != result {
                    self.state = TransitionState::Out(result, 0.0);
                }
                NextScene::Continue
            },
            TransitionState::Out(result, animation_time) => {
                *animation_time += dt;
                if *animation_time > Self::TOTAL_TIME {
                    return result.clone();
                };
                NextScene::Continue
            }
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::DrawArgs) {
        self.scene.draw(context, assets, args);
        match self.state {
            TransitionState::In(animation_time) => {
                Self::draw_box_in(context, animation_time);
            },
            TransitionState::Running => {},
            TransitionState::Out(_, animation_time) => {
                Self::draw_box_out(context, animation_time);
            },
        }
    }
    fn click(&mut self, point: point::Point<i32>) -> bool {
        self.scene.click(point)
    }
    fn returned_into(&mut self, object: super::Object) {
        self.reset();
        self.scene.returned_into(object)
    }
    fn called_into(&mut self, object: super::Object) {
        self.reset();
        self.scene.called_into(object)
    }
    fn jumped_into(&mut self, object: super::Object) {
        self.reset();
        self.scene.jumped_into(object)
    }
}