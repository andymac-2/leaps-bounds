use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::cell::{Cell, CellCursor, Direction};
use crate::cow::{Command, Cows};
use crate::js_ffi::KeyboardState;
use crate::util::clamp;
use crate::{Assets, Context2D, Point};

#[derive(Deserialize, Serialize, Debug, Clone)]
struct LevelState {
    board: Board,
    cows: Cows,
}
impl LevelState {
    fn new() -> Self {
        LevelState {
            board: Board::new(Cell::Empty),
            cows: Cows::new(
                0,
                vec![
                    (Point(3, 3), Direction::Right, vec![]),
                    (Point(10, 10), Direction::Right, vec![2]),
                    (Point(10, 5), Direction::Right, vec![]),
                ],
            ),
        }
    }

    fn command(&mut self, command: Command) {
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
        self.board
            .draw(context, &assets.blocks, Point(0, 0), Point(32, 16));
        self.cows
            .draw(context, &assets.sprites, &old_state.cows, anim_progress);
    }
}

#[derive(Debug, Clone, Copy)]
enum TimeDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone)]
pub struct Level {
    state: LevelState,
    old_states: Vec<LevelState>,
    time_direction: TimeDirection,
    animation_time: f64,
    cursor: CellCursor,
}

// INVARIANT:
impl Level {
    pub fn new() -> Self {
        Level {
            state: LevelState::new(),
            old_states: Vec::new(),
            time_direction: TimeDirection::Forward,
            animation_time: 0.0,
            cursor: CellCursor::new(),
        }
    }

    pub fn left_click(&mut self, point: Point<i32>) {
        self.state.board.left_click(point, self.cursor);
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

        if self.keyboard_event(keyboard_state, "KeyU") {
            self.pop_state();
            self.animation_time = 0.0;
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
            let mut new_state = self.state.clone();
            new_state.command(command);

            self.push_state(new_state);

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

    fn push_state(&mut self, state: LevelState) {
        match self.time_direction {
            TimeDirection::Forward => {
                let old_state = std::mem::replace(&mut self.state, state);
                self.old_states.push(old_state);
            }
            // if the time direction is backward, the current state is actually
            // on the top of the old_states stack, rather than in self.stack
            TimeDirection::Backward => {
                assert!(!self.old_states.is_empty());
                self.state = state;
                self.time_direction = TimeDirection::Forward;
            }
        }
    }

    fn pop_state(&mut self) {
        match self.time_direction {
            // The current state is actually on the top of the old_state stack
            // rather than in self.state when the time direction is backwards.
            // So all that's required to pop the state is to change the time
            // direction
            TimeDirection::Forward => {
                if !self.old_states.is_empty() {
                    self.time_direction = TimeDirection::Backward;
                }
            }
            TimeDirection::Backward => self.state = self.old_states.pop().unwrap(),
        }
        if self.old_states.is_empty() {
            self.time_direction = TimeDirection::Forward;
        }
    }

    fn old_state(&self) -> &LevelState {
        match self.time_direction {
            TimeDirection::Forward => self.old_states.last().unwrap_or(&self.state),
            TimeDirection::Backward => &self.state,
        }
    }

    fn current_state(&self) -> &LevelState {
        match self.time_direction {
            TimeDirection::Forward => &self.state,
            TimeDirection::Backward => self.old_states.last().unwrap(),
        }
    }

    pub fn draw(&self, context: &Context2D, assets: &Assets) {
        let anim_progress = clamp(self.animation_time / Level::ANIMATION_TIME, 0.0, 1.0);
        self.current_state()
            .draw(context, assets, self.old_state(), anim_progress);
    }

    const ANIMATION_TIME: f64 = 100.0;
    const COOLDOWN_TIME: f64 = 50.0;
}
