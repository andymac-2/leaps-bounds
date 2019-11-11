use serde::{Deserialize, Serialize};

use crate::console_log;
use crate::direction::Direction;
use crate::js_ffi::KeyboardState;
use crate::sprite_sheet::SpriteSheet;
use crate::state_stack::StateStack;
use crate::util::{clamp, with_saved_context};
use crate::{Assets, Context2D, Point};

mod cow;
mod cell;
mod board;

use cow::{Command, Cows};
use board::Board;
use cell::{CellCursor, GroundCell, OverlayCell};

#[derive(Copy, Clone, Debug)]
enum SuccessState {
    Failed = 0,
    Running = 1,
    Succeeded = 2,
}
impl SuccessState {
    fn combine(&mut self, other: SuccessState) {
        match (*self, other) {
            (SuccessState::Failed, _) | (_, SuccessState::Failed) => {
                *self = SuccessState::Failed;
            },
            (SuccessState::Running, _) | (_, SuccessState::Running) => {
                *self = SuccessState::Running;
            },
            _ => {
                *self = SuccessState::Succeeded;
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LevelState {
    board: Board,
    cows: Cows,
    animation_frame: u8,
}
impl LevelState {
    fn new() -> Self {
        LevelState {
            board: Board::new(GroundCell::Empty, OverlayCell::Empty),
            cows: Cows::new(
                0,
                vec![
                    (Point(3, 3), Direction::Right, vec![]),
                    (Point(10, 10), Direction::Right, vec![2]),
                    (Point(10, 5), Direction::Right, vec![]),
                ],
            ),
            animation_frame: LevelState::INITIAL_ANIMATION_FRAME,
        }
    }

    pub fn log_level(&self) {
        console_log!("{}", ron::ser::to_string(self).unwrap());
    }

    fn success_state(&self) -> SuccessState {
        self.cows.success_state(&self.board)
    }

    fn left_click(&mut self, point: Point<i32>, cursor: CellCursor) {
        self.board.left_click(point, cursor);
    }

    fn command(&mut self, command: Command) {
        self.animation_frame = (self.animation_frame + 1) % LevelState::TOTAL_ANIMATION_FRAMES;
        self.cows.command_player(&mut self.board, command);
    }

    fn draw(
        &self,
        context: &Context2D,
        assets: &Assets,
        old_state: &LevelState,
        anim_progress: f64,
    ) {
        // TODO variable dimension/ofset of tiles.
        self.board.draw_ground(
            context,
            &assets.blocks,
            Point(0, 0),
            Point(Level::LEVEL_WIDTH, Level::LEVEL_HEIGHT),
        );
        self.cows.draw(
            context,
            &assets.sprites,
            &old_state.cows,
            anim_progress,
            self.animation_frame,
        );
        self.board.draw_overlay(
            context,
            &assets.blocks,
            Point(0, 0),
            Point(Level::LEVEL_WIDTH, Level::LEVEL_HEIGHT),
        );
    }

    const TOTAL_ANIMATION_FRAMES: u8 = 4;
    const INITIAL_ANIMATION_FRAME: u8 = 0;
}

#[derive(Debug, Clone)]
pub struct Level {
    states: StateStack<LevelState>,
    animation_time: f64,
    cursor: CellCursor,
}

impl Level {
    const LEVEL_WIDTH: i32 = 32;
    const LEVEL_HEIGHT: i32 = 16;
    pub fn new() -> Self {
        Level {
            states: StateStack::new(LevelState::new()),
            animation_time: 0.0,
            cursor: CellCursor::new(),
        }
    }

    pub fn left_click(&mut self, point: Point<i32>) {
        let cursor = self.cursor;
        self.states.current_state_mut().left_click(point, cursor);
    }
    pub fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) {
        self.animation_time += dt;

        if keyboard_state.is_pressed("KeyT") {
            self.cursor.increment_type();
        }
        if keyboard_state.is_pressed("KeyR") {
            self.cursor.increment_direction();
        }
        if keyboard_state.is_pressed("KeyC") {
            self.cursor.increment_colour();
        }

        if keyboard_state.is_pressed("KeyL") {
            self.states.current_state().log_level();
        }

        if self.keyboard_event(keyboard_state, "KeyU") {
            self.states.pop_state();
            self.animation_time = 0.0;
            return;
        }

        let opt_command = if self.keyboard_event(keyboard_state, "ArrowUp") {
            Some(Command::Walk(Direction::Up))
        } else if self.keyboard_event(keyboard_state, "ArrowRight") {
            Some(Command::Walk(Direction::Right))
        } else if self.keyboard_event(keyboard_state, "ArrowDown") {
            Some(Command::Walk(Direction::Down))
        } else if self.keyboard_event(keyboard_state, "ArrowLeft") {
            Some(Command::Walk(Direction::Left))
        } else if self.keyboard_event(keyboard_state, "Space") {
            Some(Command::Halt)
        } else {
            None
        };

        if let Some(command) = opt_command {
            let mut current_state = self.states.current_state().clone();
            current_state.command(command);

            self.states.push_state(current_state);

            self.animation_time = 0.0;
        }
    }

    fn is_finished_animating(&self) -> bool {
        self.animation_time > Level::ANIMATION_TIME + Level::COOLDOWN_TIME
    }
    fn keyboard_event(&self, keyboard_state: &KeyboardState, code: &str) -> bool {
        if self.is_finished_animating() {
            return keyboard_state.is_held(code);
        }
        keyboard_state.is_pressed(code)
    }

    pub fn draw(&self, context: &Context2D, assets: &Assets) {
        let anim_progress = clamp(self.animation_time / Level::ANIMATION_TIME, 0.0, 1.0);

        let canvas_width = f64::from(Level::LEVEL_WIDTH) * f64::from(SpriteSheet::STANDARD_WIDTH);
        let canvas_height =
            f64::from(Level::LEVEL_HEIGHT) * f64::from(SpriteSheet::STANDARD_HEIGHT);

        with_saved_context(context, |ctx| {
            ctx.set_fill_style(&wasm_bindgen::JsValue::from_str("rgb(113, 46, 25)"));
            ctx.fill_rect(0.0, 0.0, canvas_width, canvas_height);

            self.states.current_state().draw(
                context,
                assets,
                self.states.last_state(),
                anim_progress,
            );
            ctx.scale(2.0, 2.0).unwrap();
            self.cursor.draw(context, &assets.blocks)
        });
    }

    const ANIMATION_TIME: f64 = 100.0;
    const COOLDOWN_TIME: f64 = 50.0;
}
