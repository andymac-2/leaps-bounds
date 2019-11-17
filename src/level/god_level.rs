use crate::{component, Context2D, Assets};
use crate::point::Point;
use crate::util::clamp;

use super::{LevelState, SuccessState};
use super::cell::{CellPalette, CellType};
use super::cow_level::CowLevel;

pub struct GodLevel {
    cell_palette: CellPalette<CellType>,
    initial_state: LevelState,
    running_state: GodLevelStatus,
    speed: f64,
}
impl GodLevel {
    const MIN_SPEED: f64 = 500.0;
    fn new() -> Self {
        GodLevel {
            cell_palette: CellPalette::new(CellType::full_palette()),
            initial_state: LevelState::new(),
            running_state: GodLevelStatus::new(),
            speed: Self::MIN_SPEED,
        }
    }
    fn step(&mut self, dt: f64) {
        self.running_state.step(dt);
    }
    fn toggle_pause(&mut self) {
        if self.running_state.is_stopped() {
            self.running_state.start(self.initial_state.clone(), self.speed)
        }
        else {
            self.running_state.toggle_pause()
        }
    }
    fn is_success(&self) -> bool {
        self.running_state.is_success()
    }
}
impl component::Component for GodLevel {
    type Args = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool{
        if !self.in_boundary(point) {
            return false;
        }

        if !self.running_state.is_stopped() {
            return false;
        }

        let value = self.cell_palette.value();
        self.initial_state.set_cell_at_point(point, value);
        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        if self.running_state.is_stopped() {
            // draw play button
            unimplemented!();
            // draw stop button
            unimplemented!();
            // draw initial state
            self.initial_state.draw(context, assets, &self.initial_state, 0.0);
        }
        else {
            // draw play/pause button
            unimplemented!();
            // draw stop button
            unimplemented!();
            //draw level
            self.running_state.draw(context, assets, ());
        }
        self.cell_palette.draw(context, assets, ());
    }
}

// no invariants, all states are valid.
#[derive(Clone, Debug)]
enum GodLevelStatus {
    Stopped,
    Paused(GodLevelRunningState),
    Playing(GodLevelRunningState),
}
impl GodLevelStatus {
    fn new() -> Self {
        Self::Stopped
    }
    fn step(&mut self, dt: f64) {
        match *self {
            Self::Stopped => {},
            Self::Paused(_) => {},
            Self::Playing(ref mut state) => {
                state.step(dt);
            }
        }
    }
    fn stop(&mut self) {
        *self = Self::Stopped;
    }
    fn start(&mut self, state: LevelState, speed: f64) {
        assert!(self.is_stopped());
        *self = Self::Playing(GodLevelRunningState::new(state, speed));
    }
    fn toggle_pause(&mut self) {
        let status = std::mem::replace(self, Self::Stopped);
        *self = match status {
            Self::Stopped => panic!("GodLevelStatus should be playing or paused"),
            Self::Playing(state) => Self::Paused(state),
            Self::Paused(state) => Self::Playing(state),
        }
    }
    fn is_success(&self) -> bool {
        match self {
            Self::Stopped => false,
            Self::Paused(state) | Self::Playing(state) => {
                unimplemented!()
            }
        }
    }
    fn is_stopped(&self) -> bool {
        match self {
            Self::Stopped => true,
            _ => false,
        }
    }
}
impl component::Component for GodLevelStatus {
    type Args= ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        match self {
            Self::Stopped => {}
            Self::Playing(state) | Self::Paused(state) => {
                state.draw(context, assets, ());
            }
        }
    }
}

#[derive(Clone, Debug)]
struct GodLevelRunningState {
    current_state: LevelState,
    old_state: LevelState,
    animation_time: f64,
    speed: f64,
}
impl GodLevelRunningState {
    fn new(initial_state: LevelState, speed: f64) -> Self {
        GodLevelRunningState {
            current_state: initial_state.clone(),
            old_state: initial_state,
            animation_time: speed,
            speed,
        }
    }
    fn success_state(&self) -> SuccessState {
        self.current_state.success_state()
    }
    fn step(&mut self, dt: f64) {
        self.animation_time += dt;
        if self.animation_time < self.speed {
            return;
        }

        self.animation_time = 0.0;
        self.old_state.clone_from(&self.current_state);
        self.current_state.auto();
    }
}
impl component::Component for GodLevelRunningState {
    type Args = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = clamp(self.animation_time / self.speed, 0.0, 1.0);
        self.current_state.draw(context, assets, &self.old_state, anim_progress);
    }
}