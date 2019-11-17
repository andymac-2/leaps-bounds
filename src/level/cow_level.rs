use crate::{component, SpriteSheet, Context2D, Assets, util, js_ffi};
use crate::point::Point;
use crate::scene;
use crate::scene::{NextScene, Object};

use super::{StateStack, LevelState, Level, SuccessState};
use super::cell::{CellType, CellPalette};

#[derive(Debug, Clone)]
pub struct CowLevel {
    states: StateStack<LevelState>,
    animation_time: f64,
    palette: CellPalette<CellType>,
}

impl CowLevel {
    pub const LEVEL_WIDTH: i32 = 32;
    pub const LEVEL_HEIGHT: i32 = 16;
    pub const ANIMATION_TIME: f64 = 100.0;
    pub const COOLDOWN_TIME: f64 = 50.0;
    pub const BOUNDING_RECT: component::Rect = component::Rect {
        top_left: Point(0, 0),
        dimensions: Point(
            Self::LEVEL_WIDTH * SpriteSheet::STANDARD_WIDTH, 
            Self::LEVEL_HEIGHT * SpriteSheet::STANDARD_HEIGHT,
        ),
    };
    fn from_state(state: LevelState) -> Self {
        CowLevel {
            states: StateStack::new(state),
            animation_time: 0.0,
            palette: CellPalette::new(CellType::full_palette()),
        }
    }
    pub fn from_str (string: &'static str) -> Self {
        CowLevel::from_state(ron::de::from_str::<LevelState>(string).unwrap())
    }
    pub fn new() -> Self {
        CowLevel {
            states: StateStack::new(LevelState::new()),
            animation_time: 0.0,
            palette: CellPalette::new(CellType::full_palette()),
        }
    }
    fn purge_states(&mut self) {
        self.states.purge_states();
    }
}
impl Level for CowLevel {
    fn is_finished_animating(&self) -> bool {
        self.animation_time > CowLevel::ANIMATION_TIME + CowLevel::COOLDOWN_TIME
    }
}
impl component::Component for CowLevel {
    type Args = ();
    fn bounding_rect(&self) -> component::Rect {
        Self::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if self.palette.click(point) {
            return true;
        }

        let value = self.palette.value();
        self.states.current_state_mut().set_cell_at_point(point, value);

        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = util::clamp(self.animation_time / CowLevel::ANIMATION_TIME, 0.0, 1.0);

        self.fill_bg(context, super::BG_FILL);

        util::with_saved_context(context, || {
            self.states.current_state().draw(
                context,
                assets,
                self.states.last_state(),
                anim_progress,
            );
            self.palette.draw(context, assets, ())
        });
    }
}
impl scene::Scene for CowLevel {
    fn called_into(&mut self, _object: Object) {
        self.purge_states();
    }
    fn step(&mut self, dt: f64, keyboard_state: &js_ffi::KeyboardState) -> NextScene {
        self.animation_time += dt;

        // undo and redo should still be possible after failure
        if self.keyboard_event(keyboard_state, "KeyR") {
            self.purge_states();
        }

        if self.keyboard_event(keyboard_state, "KeyU") {
            self.states.pop_state();
            self.animation_time = 0.0;
            return NextScene::Continue;
        }

        if keyboard_state.is_pressed("KeyL") {
            self.states.current_state().log_level();
        }

        // block character movement n success or failure.
        match self.states.current_state().success_state() {
            SuccessState::Succeeded => {
                if !self.is_finished_animating() {
                    return NextScene::Continue;
                }
                return NextScene::Return(Object::Bool(true));
            },
            SuccessState::Failed => {
                assert!(self.is_finished_animating());
                return NextScene::Continue;
            },
            SuccessState::Running => {}
        };

        if let Some(command) = self.get_keyboard_command(keyboard_state) {
            let mut current_state = self.states.current_state().clone();
            current_state.command(command.into());

            self.states.push_state(current_state);

            self.animation_time = 0.0;
        };

        NextScene::Continue
    }
}