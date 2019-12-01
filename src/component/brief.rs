use crate::{Context2D, Assets, Point, util, KeyboardState};
use crate::component::{Component, Rect, NextScene};

#[derive(Clone, Debug)]
pub struct Brief<T> {
    description: &'static str,
    is_expanded: bool,
    scene: T,
}
impl<T: Component> Brief<T> {
    pub fn new(description: &'static str, scene: T) -> Self {
        Brief {
            description,
            is_expanded: false,
            scene,
        }
    }

    fn get_button_rect(&self) -> Rect {
        self.bounding_rect().shrink_bottom_right(Rect::TWO_BY_TWO)
    }

    const BG_PAPER: Rect = Rect::indexed(Point(1, 0), Rect::FOUR_BY_TWO);
    const PAPER_ICON: Rect = Rect::indexed(Point(0, 6), Rect::TWO_BY_TWO);
    const TOP_MARGIN: f64 = 60.0;
    const DESCRIPTION_TOP: f64 = 105.0;
    const LEFT_MARGIN: f64 = 70.0;
    const LINE_HEIGHT: f64 = 16.0;
}
impl<T: Component> Component for Brief<T> {
    type DrawArgs = T::DrawArgs;
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::DrawArgs) {
        if !self.is_expanded {
            self.scene.draw(context, assets, args);

            let bottom_left = self.get_button_rect();
            assets.misc.draw_with_rect(context, &Self::PAPER_ICON, &bottom_left);
            return;
        }

        let bounding_rect = self.bounding_rect();
        let centre = f64::from(bounding_rect.centre().x());

        util::with_saved_context(context, || {
            assets.misc.draw_with_rect(context, &Self::BG_PAPER, &bounding_rect);

            let black = wasm_bindgen::JsValue::from_str("black");

            context.set_font("25px KongText");
            context.set_text_align("center");
            context.set_fill_style(&black);

            context
                .fill_text("Brief:", centre, Self::TOP_MARGIN)
                .unwrap();

            context.set_font("15px KongText");
            context.set_text_align("left");

            let left_margin = f64::from(bounding_rect.top_left.x()) + Self::LEFT_MARGIN;
            let mut baseline = f64::from(bounding_rect.top_left.y()) + Self::DESCRIPTION_TOP;
        
            for line in self.description.lines() {
                context
                    .fill_text(line, left_margin, baseline)
                    .unwrap();
        
                baseline += Self::LINE_HEIGHT;
            }
        })
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if self.is_expanded {
            self.is_expanded = false;
            return true;
        }

        if self.get_button_rect().inside(point) {
            self.is_expanded = true;
            return true;
        }

        self.scene.click(point)
    }
    fn bounding_rect(&self) -> Rect {
        self.scene.bounding_rect()
    }
    fn step(&mut self, dt: f64, keyboard: &KeyboardState) -> NextScene {
        if self.is_expanded {
            if keyboard.is_pressed("Space") || keyboard.is_pressed("Enter") {
                self.is_expanded = false;
            }
            return NextScene::Continue;
        };
        
        self.scene.step(dt, keyboard)
    }
    fn returned_into(&mut self, object: super::Object) {
        self.scene.returned_into(object)
    }
    fn called_into(&mut self, object: super::Object) {
        self.is_expanded = true;
        self.scene.called_into(object)
    }
    fn jumped_into(&mut self, object: super::Object) {
        self.is_expanded = true;
        self.scene.jumped_into(object)
    }
}