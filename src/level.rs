use serde::{Deserialize, Serialize};

use crate::console_log;
use crate::direction::Direction;
use crate::js_ffi::KeyboardState;
use crate::state_stack::StateStack;
use crate::{Assets, Context2D, Point};

mod board;
pub mod cell;
mod cow;
pub mod cow_level;
pub mod god_level;
pub mod overworld_level;

use board::Board;
use cell::{CellType, GroundCell, OverlayCell, PaletteResult};
use cow::{Command, CowSprite, Cows};
use cow_level::CowLevel;

// green.
const BG_FILL: &str = "#669238";

#[derive(Clone, Debug)]
struct NotEnoughInputSpace;

#[derive(Debug, Clone, Copy)]
pub enum KeyboardCommand {
    Direction(Direction),
    Space,
}
impl KeyboardCommand {
    fn is_space(&self) -> bool {
        match self {
            Self::Space => true,
            _ => false,
        }
    }
}

trait Level {
    fn is_finished_animating(&self) -> bool;
    fn get_keyboard_command(&self, keyboard_state: &KeyboardState) -> Option<KeyboardCommand> {
        if self.keyboard_event(keyboard_state, &["ArrowUp", "KeyW"]) {
            Some(KeyboardCommand::Direction(Direction::Up))
        } else if self.keyboard_event(keyboard_state, &["ArrowRight", "KeyD"]) {
            Some(KeyboardCommand::Direction(Direction::Right))
        } else if self.keyboard_event(keyboard_state, &["ArrowDown", "KeyS"]) {
            Some(KeyboardCommand::Direction(Direction::Down))
        } else if self.keyboard_event(keyboard_state, &["ArrowLeft", "KeyA"]) {
            Some(KeyboardCommand::Direction(Direction::Left))
        } else if self.keyboard_event(keyboard_state, &["Space", "Enter"]) {
            Some(KeyboardCommand::Space)
        } else {
            None
        }
    }
    fn keyboard_event(&self, keyboard_state: &KeyboardState, codes: &[&str]) -> bool {
        for code in codes.iter() {
            if self.is_finished_animating() && keyboard_state.is_held(code) {
                return true;
            }
            if keyboard_state.is_pressed(code) {
                return true;
            }
        }
        false
    }
}

pub trait Pasture<C> {
    fn get_pasture_cell(&self, point: Point<i32>) -> &C;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SuccessState {
    Failed = 0,
    Running = 1,
    Succeeded = 2,
}
impl SuccessState {
    fn is_running(&self) -> bool {
        match self {
            SuccessState::Running => true,
            _ => false,
        }
    }
    fn is_success(&self) -> bool {
        match self {
            SuccessState::Succeeded => true,
            _ => false,
        }
    }
    fn combine(&mut self, other: SuccessState) {
        match (*self, other) {
            (SuccessState::Failed, _) | (_, SuccessState::Failed) => {
                *self = SuccessState::Failed;
            }
            (SuccessState::Running, _) | (_, SuccessState::Running) => {
                *self = SuccessState::Running;
            }
            _ => {
                *self = SuccessState::Succeeded;
            }
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
                    (Point(24, 4), Direction::Right, CowSprite::Brown, vec![1]),
                    (Point(8, 12), Direction::Right, CowSprite::White, vec![]),
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

    fn get_overlay_cell_at_point(&mut self, point: Point<i32>) -> OverlayCell {
        self.board.get_overlay_cell_at_point(point)
    }
    fn set_cell_at_point(&mut self, point: Point<i32>, cell_type: PaletteResult<CellType>) {
        self.board.set_cell_at_point(point, cell_type);
    }

    fn set_inputs(&mut self, inputs: &[cell::Colour]) -> Result<(), NotEnoughInputSpace> {
        self.board.set_inputs(inputs)
    }
    fn get_outputs(&self) -> Vec<cell::Colour> {
        self.board.get_outputs()
    }

    fn auto(&mut self) {
        self.command(Command::Auto);
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
            Point(CowLevel::LEVEL_WIDTH, CowLevel::LEVEL_HEIGHT),
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
            Point(CowLevel::LEVEL_WIDTH, CowLevel::LEVEL_HEIGHT),
        );
    }

    const TOTAL_ANIMATION_FRAMES: u8 = 4;
    const INITIAL_ANIMATION_FRAME: u8 = 0;
}
