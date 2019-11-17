use serde::{Deserialize, Serialize};

use crate::console_log;
use crate::direction::Direction;
use crate::js_ffi::KeyboardState;
use crate::state_stack::StateStack;
use crate::{Assets, Context2D, Point};

mod cow;
mod cell;
mod board;
mod god_level;
pub mod cow_level;
pub mod overworld_level;

use cow::{Command, Cows, CowSprite};
use cow_level::CowLevel;
use board::{Board};
use cell::{PaletteResult, GroundCell, OverlayCell, CellType};

// green.
const BG_FILL: &str = "#669238";

#[derive(Debug, Clone, Copy)]
pub enum KeyboardCommand {
    Direction(Direction),
    Space,
}
impl KeyboardCommand {
    fn is_space (&self) -> bool {
        match self {
            Self::Space => true,
            _ => false,
        }
    }
}

trait Level {

    fn is_finished_animating(&self) -> bool;
    fn get_keyboard_command(&self, keyboard_state: &KeyboardState) -> Option<KeyboardCommand> {
        if self.keyboard_event(keyboard_state, "ArrowUp") {
            Some(KeyboardCommand::Direction(Direction::Up))
        } else if self.keyboard_event(keyboard_state, "ArrowRight") {
            Some(KeyboardCommand::Direction(Direction::Right))
        } else if self.keyboard_event(keyboard_state, "ArrowDown") {
            Some(KeyboardCommand::Direction(Direction::Down))
        } else if self.keyboard_event(keyboard_state, "ArrowLeft") {
            Some(KeyboardCommand::Direction(Direction::Left))
        } else if self.keyboard_event(keyboard_state, "Space") {
            Some(KeyboardCommand::Space)
        } else {
            None
        }
    }
    fn keyboard_event(&self, keyboard_state: &KeyboardState, code: &str) -> bool {
        if self.is_finished_animating() {
            return keyboard_state.is_held(code);
        }
        keyboard_state.is_pressed(code)
    }
}

trait Pasture<C> {
    fn get_pasture_cell(&self, point: Point<i32>) -> &C;
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SuccessState {
    Failed = 0,
    Running = 1,
    Succeeded = 2,
}
impl SuccessState {
    fn is_running (&self) -> bool{
        match self {
            SuccessState::Running => true,
            _ => false,
        }
    }
    fn is_success (&self) -> bool {
        match self {
            SuccessState::Succeeded => true,
            _ => false,
        }
    }
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
                    (Point(3, 3), Direction::Right, CowSprite::Grey, vec![]),
                    (Point(10, 10), Direction::Right, CowSprite::White, vec![2]),
                    (Point(10, 5), Direction::Right, CowSprite::White, vec![]),
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

    fn set_cell_at_point(&mut self, point: Point<i32>, cell_type: PaletteResult<CellType>) {
        self.board.set_cell_at_point(point, cell_type);
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