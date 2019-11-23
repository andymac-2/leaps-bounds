use crate::component::{Translation, NextScene, Object};
use crate::point::Point;
use crate::util::clamp;
use crate::{component, Assets, Context2D, SpriteSheet, KeyboardState};

use super::cell::{Colour, CellPalette, CellType, CellGraphic, PaletteResult, cell_cursor};
use super::cow_level::CowLevel;
use super::{LevelState, SuccessState, NotEnoughInputSpace};


#[derive(Clone, Debug)]
pub struct Test {
    input: Vec<Colour>,
    output: TestResult,
}
impl Test {
    pub fn new(input: Vec<Colour>, output: TestResult) -> Test {
        Test {input, output}
    }
}
#[derive(Clone, Debug)]
pub enum TestResult {
    Reject,
    Accept,
    AcceptWith(Vec<Colour>),
}

pub struct GodLevel {
    control_panel: ControlPanel,
    initial_state: LevelState,
    running_state: GodLevelStatus,
    speed: f64,
    tests: Vec<Test>,
    current_test: usize,
}
impl GodLevel {
    const MIN_SPEED: f64 = 500.0;
    pub fn new(tests: Vec<Test>) -> Self {
        let palette = CellPalette::new(CellType::full_palette());
        GodLevel {
            control_panel: ControlPanel::new(palette),
            initial_state: LevelState::new(),
            running_state: GodLevelStatus::new(),
            speed: Self::MIN_SPEED,
            tests,
            current_test: 0,
        }
    }
    fn is_success(&self) -> bool {
        self.current_test >= self.tests.len()
    }
    fn get_current_test(&self) -> &Test {
        &self.tests[self.current_test]
    }
    fn next_test(&mut self) {
        let mut state = self.initial_state.clone();
        let test = self.get_current_test().clone();

        self.running_state.stop();

        if state.set_inputs(&test.input).is_err() {
            crate::console_log!("not enough room");
            return;
        }

        self.running_state.start(state, test.output, self.speed);
        self.current_test += 1;
    }
    fn control_button_press(&mut self, button: ControlButton) {
        match button {
            ControlButton::Play => {
                if !self.running_state.is_stopped() {
                    self.running_state.play();
                    return;
                }
                self.current_test = 0;
                self.next_test();
            },
            ControlButton::Stop => {
                self.current_test = 0;
                self.running_state.stop()
            },
            ControlButton::Pause => self.running_state.pause(),
        }
    }
}
impl component::Component for GodLevel {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if self.control_panel.click(point) {
            self.control_button_press(self.control_panel.last_press());
            return true;
        }

        if !self.running_state.is_stopped() {
            return false;
        }

        let value = self.control_panel.cell_palette_value();
        self.initial_state.set_cell_at_point(point, value);
        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.fill_bg(context, super::BG_FILL);

        if self.running_state.is_stopped() {
            self.initial_state
                .draw(context, assets, &self.initial_state, 0.0);
        } 
        else {
            self.running_state.draw(context, assets, ());
        }

        self.control_panel.fill_bg(context, cell_cursor::BG_COLOUR);
        self.control_panel.draw(context, assets, ());
    }
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene{
        match self.running_state.step(dt, keyboard_state) {
            NextScene::Continue => NextScene::Continue,
            NextScene::Return(Object::Bool(success)) => {
                if !success {
                    //TODO make a failure screen.
                    self.running_state = GodLevelStatus::Stopped;
                    NextScene::Continue
                }
                else if self.is_success() {
                    NextScene::Return(Object::Bool(true))
                }
                else {
                    crate::here!();
                    NextScene::Continue
                }
            },
            _ => unreachable!(),
        }
    }
}

// no invariants, all states are valid.
#[derive(Clone, Debug)]
enum GodLevelStatus {
    Stopped,
    Paused(Box<GodLevelRunningState>),
    Playing(Box<GodLevelRunningState>),
}
impl GodLevelStatus {
    fn new() -> Self {
        Self::Stopped
    }
    fn stop(&mut self) {
        *self = Self::Stopped;
    }
    fn start(&mut self, state: LevelState, target: TestResult, speed: f64) {
        assert!(self.is_stopped());
        *self = Self::Playing(Box::new(GodLevelRunningState::new(state, target, speed)));
    }
    fn pause (&mut self) {
        let status = std::mem::replace(self, Self::Stopped);
        *self = match status {
            Self::Stopped => Self::Stopped,
            Self::Playing(state) => Self::Paused(state),
            Self::Paused(state) => Self::Paused(state),
        }
    }
    fn play (&mut self) {
        let status = std::mem::replace(self, Self::Stopped);
        *self = match status {
            Self::Stopped => panic!("Play used on stopped variant. Use start instead."),
            Self::Playing(state) => Self::Playing(state),
            Self::Paused(state) => Self::Playing(state),
        }
    }
    fn is_complete(&self) -> bool {
        match self {
            Self::Stopped => false,
            Self::Paused(state) | Self::Playing(state) => state.is_complete(),
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
    type DrawArgs = ();
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
    fn step(&mut self, dt: f64, _keyboard: &KeyboardState) -> NextScene {
        match *self {
            Self::Stopped => NextScene::Continue,
            Self::Paused(_) => NextScene::Continue,
            Self::Playing(ref mut state) => {
                state.step(dt);
                if state.is_complete() {
                    NextScene::Return(Object::Bool(state.test_passed()))
                }
                else {
                    NextScene::Continue
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct GodLevelRunningState {
    current_state: LevelState,
    old_state: LevelState,
    target: TestResult,
    animation_time: f64,
    speed: f64,
}
impl GodLevelRunningState {
    fn new(initial_state: LevelState, target: TestResult, speed: f64) -> Self {
        GodLevelRunningState {
            current_state: initial_state.clone(),
            old_state: initial_state,
            target,
            animation_time: speed,
            speed,
        }
    }
    fn test_passed(&self) -> bool {
        match (&self.target, self.current_state.success_state()) {
            (TestResult::Reject, SuccessState::Failed) => true,
            (TestResult::Accept, SuccessState::Succeeded) => true,
            (TestResult::AcceptWith(colour_string), SuccessState::Succeeded) => {
                let output = self.current_state.get_outputs();
                &output == colour_string
            },
            (_, _) => false,
        }
    }

    /// is complete if all cows are in a success zone or one is in a failure zone.
    fn is_complete(&self) -> bool {
        !self.current_state.success_state().is_running() && self.animation_time > self.speed
    }

    fn step(&mut self, dt: f64) {
        self.animation_time += dt;
        if self.animation_time < self.speed || !self.current_state.success_state().is_running() {
            return;
        }

        self.animation_time = 0.0;
        self.old_state.clone_from(&self.current_state);
        self.current_state.auto();
    }
}
impl component::Component for GodLevelRunningState {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = clamp(self.animation_time / self.speed, 0.0, 1.0);
        self.current_state
            .draw(context, assets, &self.old_state, anim_progress);
    }
}

#[derive(Clone, Debug, Copy)]
pub enum ControlButton {
    Play,
    Pause,
    Stop,
}
#[derive(Clone, Debug)]
struct ControlPanel {
    cell_palette: Translation<CellPalette<CellType>>,
    last_press: ControlButton,
}
impl ControlPanel {
    const HALF_HEIGHT: i32 = SpriteSheet::STANDARD_HEIGHT / 2;
    const HALF_WIDTH: i32 = SpriteSheet::STANDARD_WIDTH / 2;
    const PALETTE_OFFSET: Point<i32> = Point(0, Self::HALF_HEIGHT * 3);
    const PLAY_BUTTON: CellGraphic = CellGraphic::new(
        Point(Self::HALF_WIDTH, Self::HALF_HEIGHT),
        Point(15, 0),
    );
    const PAUSE_BUTTON: CellGraphic = CellGraphic::new(
        Point(Self::HALF_WIDTH * 3, Self::HALF_HEIGHT),
        Point(14, 0),
    );
    const STOP_BUTTON: CellGraphic = CellGraphic::new(
        Point(Self::HALF_WIDTH * 7, Self::HALF_HEIGHT),
        Point(13, 0),
    );
    const CONTROL_DIMENSIONS: component::Rect = component::Rect {
        top_left: Point(0, 0),
        dimensions: Point(Self::HALF_WIDTH * 10, Self::HALF_HEIGHT * 3),
    };

    fn new(cell_palette: CellPalette<CellType>) -> Self {
        ControlPanel {
            cell_palette: Translation::new(Self::PALETTE_OFFSET, cell_palette),
            last_press: ControlButton::Pause,
        }
    }
    fn cell_palette_value(&self) -> PaletteResult<CellType> {
        self.cell_palette.value()
    }
    fn last_press(&self) -> ControlButton {
        self.last_press
    }
}
impl component::Component for ControlPanel {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        Self::CONTROL_DIMENSIONS.combine(&self.cell_palette.bounding_rect())
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if Self::PLAY_BUTTON.in_boundary(point) {
            self.last_press = ControlButton::Play;
            return true;
        }
        if Self::PAUSE_BUTTON.in_boundary(point) {
            self.last_press = ControlButton::Pause;
            return true;
        }
        if Self::STOP_BUTTON.in_boundary(point) {
            self.last_press = ControlButton::Stop;
            return true;
        }
        
        self.cell_palette.click(point)
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        Self::PLAY_BUTTON.draw(context, assets, ());
        Self::PAUSE_BUTTON.draw(context, assets, ());
        Self::STOP_BUTTON.draw(context, assets, ());

        self.cell_palette.draw(context, assets, ());
    }
}