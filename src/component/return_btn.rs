use crate::{Context2D, Assets, KeyboardState, util, point};

use super::{NextScene, Rect};

pub struct ReturnButton<T> {
    scene: T,
    is_returning: bool,
}
impl<T> ReturnButton<T> 
where
    T: super::Component,
{
    pub fn new(scene: T) -> Self {
        ReturnButton {
            scene,
            is_returning: false,
        }
    }
    fn get_button_bounds(&self) -> Rect {
        let bounds = self.scene.bounding_rect();
        bounds.shrink_bottom_left(Rect::TWO_BY_TWO)
    }
    const BACK_BUTTON: Rect = Rect::indexed(point::Point(0, 5), Rect::TWO_BY_TWO);
}

impl<T> super::Component for ReturnButton<T>
where 
    T: super::Component,
{
    type DrawArgs = T::DrawArgs;
    fn bounding_rect(&self) -> super::Rect {
        self.scene.bounding_rect()
    }
    fn step(&mut self, dt: f64, keyboard: &KeyboardState) -> NextScene {
        if self.is_returning {
            return NextScene::Return(super::Object::Null);
        }
        self.scene.step(dt, keyboard)
    }
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::DrawArgs) {
        self.scene.draw(context, assets, args);

        let destination = self.get_button_bounds();
        assets.misc.draw_with_rect(context, &Self::BACK_BUTTON, &destination);
    }
    fn click(&mut self, point: point::Point<i32>) -> bool {
        if self.get_button_bounds().inside(point) {
            self.is_returning = true;
            return true;
        }

        self.scene.click(point)
    }
    fn returned_into(&mut self, object: super::Object) {
        self.is_returning = false;
        self.scene.returned_into(object)
    }
    fn called_into(&mut self, object: super::Object) {
        self.is_returning = false;
        self.scene.called_into(object)
    }
    fn jumped_into(&mut self, object: super::Object) {
        self.is_returning = false;
        self.scene.jumped_into(object)
    }
}