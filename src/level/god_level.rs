use crate::component::{NextScene, Object, Translation};
use crate::point::Point;
use crate::util;
use crate::{component, Assets, Context2D, KeyboardState, SpriteSheet};

use super::cell::{cell_cursor, CellGraphic, CellPalette, CellType, PaletteResult};
use super::cow_level::CowLevel;
use super::{LevelState, SuccessState};

mod test;

use test::{MetaTestResult, TestResult};
pub use test::{Test, TestTarget};

pub struct GodLevel {
    name: &'static str,
    control_panel: ControlPanel,
    initial_state: LevelState,
    running_state: GodLevelStatus,
    speed: f64,
    tests: Vec<Test>,
    current_test: usize,
}
impl GodLevel {
    const MIN_SPEED: f64 = 500.0;
    const MAX_SPEED_SCALE: f64 = 100.0;
    pub fn new(name: &'static str, tests: Vec<Test>) -> Self {
        let palette = CellPalette::new(CellType::full_palette());
        GodLevel {
            name,
            control_panel: ControlPanel::new(palette),
            initial_state: LevelState::new(),
            running_state: GodLevelStatus::new(),
            speed: 1.0,
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
        let state = self.initial_state.clone();
        let test = self.get_current_test().clone();

        self.running_state.stop();
        self.running_state.start(state, test);

        self.current_test += 1;
    }
    fn reset_tests(&mut self) {
        self.running_state.stop();
        self.current_test = 0;
    }
    fn save_state(&self) {
        let local_storage = util::get_storage();
        let state_str = ron::ser::to_string(&self.initial_state).unwrap();

        if local_storage.set_item(self.name, &state_str).is_err() {
            crate::console_error!("Could not save to local storage");
        }
    }
    fn restore_state(&mut self) {
        let local_storage = util::get_storage();

        match local_storage.get_item(self.name) {
            Err(_) => crate::console_error!("Could not access local storage"),
            Ok(None) => {},
            Ok(Some(string)) => {
                let state: LevelState = ron::de::from_str(&string).unwrap();
        
                self.initial_state = state;
                self.running_state = GodLevelStatus::new();
                self.current_test = 0;
            },
        }
    }

    fn control_button_press(&mut self, button: ControlButton) {
        match button {
            ControlButton::Play => {
                if !self.running_state.is_stopped() {
                    self.running_state.play();
                    return;
                }

                self.save_state();
                self.current_test = 0;
                self.next_test();
            }
            ControlButton::Stop => {
                self.reset_tests();
            }
            ControlButton::Pause => self.running_state.pause(),
        }
    }
}
impl component::Component for GodLevel {
    type DrawArgs = ();
    fn called_into(&mut self, _object: Object) {
        self.restore_state();
        self.reset_tests();
    }
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if self.control_panel.click(point) {
            if let Some(button) = self.control_panel.last_press() {
                self.control_button_press(button);
            }
            return true;
        }

        match &mut self.running_state {
            GodLevelStatus::Report(result) => {
                let result = result.clone();
                self.running_state.close_report(&result);
                true
            },
            GodLevelStatus::Stopped => {
                let value = self.control_panel.cell_palette_value();
                self.initial_state.set_cell_at_point(point, value);
                true
            },
            _ => false
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.fill_bg(context, super::BG_FILL);

        if self.running_state.is_drawable() {
            self.running_state.draw(context, assets, ());
        } else {
            self.initial_state
                .draw(context, assets, &self.initial_state, 0.0);
        }

        if !self.running_state.is_report() {
            self.control_panel.fill_bg(context, cell_cursor::BG_COLOUR);
            self.control_panel.draw(context, assets, ());
        }
    }
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        if keyboard_state.is_pressed("Add") || keyboard_state.is_pressed("ArrowUp") {
            self.speed += 1.0;
        }
        if keyboard_state.is_pressed("Subtract") || keyboard_state.is_pressed("ArrowDown") {
            self.speed -= 1.0;
        }
        self.speed = util::clamp(self.speed, 1.0, Self::MAX_SPEED_SCALE);

        self.running_state.step(dt * self.speed, keyboard_state);
        if self.running_state.is_succeeded() {
            if self.is_success() {
                return NextScene::Return(Object::Bool(true));
            } else {
                self.next_test();
                return NextScene::Continue;
            }
        }
        NextScene::Continue
    }
}

// no invariants, all states are valid.
#[derive(Clone, Debug)]
enum GodLevelStatus {
    Stopped,
    Paused(Test, Box<GodLevelRunningState>),
    Playing(Test, Box<GodLevelRunningState>),
    Report(MetaTestResult),
    Succeeded,
}
impl GodLevelStatus {
    fn new() -> Self {
        Self::Stopped
    }
    fn stop(&mut self) {
        *self = Self::Stopped;
    }
    fn start(&mut self, mut state: LevelState, test: Test) {
        assert!(self.is_stopped());
        if let Ok(()) = state.set_inputs(test.input()) {
            *self = Self::Playing(test, Box::new(GodLevelRunningState::new(state)));
        } else {
            let result = MetaTestResult::new(test, TestResult::NotEnoughInputSpace);
            *self = Self::Report(result);
        }
    }
    fn pause(&mut self) {
        let status = std::mem::replace(self, Self::Stopped);
        *self = match status {
            Self::Stopped => Self::Stopped,
            Self::Playing(test, state) => Self::Paused(test, state),
            Self::Paused(test, state) => Self::Paused(test, state),
            Self::Report(result) => Self::Report(result),
            Self::Succeeded => Self::Succeeded,
        }
    }
    fn play(&mut self) {
        let status = std::mem::replace(self, Self::Stopped);
        *self = match status {
            Self::Stopped => panic!("Play used on stopped variant. Use start instead."),
            Self::Playing(test, state) => Self::Playing(test, state),
            Self::Paused(test, state) => Self::Playing(test, state),
            Self::Report(result) => Self::Report(result),
            Self::Succeeded => Self::Succeeded,
        }
    }
    fn is_succeeded(&self) -> bool {
        match self {
            Self::Succeeded => true,
            _ => false,
        }
    }
    fn is_stopped(&self) -> bool {
        match self {
            Self::Stopped => true,
            _ => false,
        }
    }
    fn is_drawable(&self) -> bool {
        match self {
            Self::Stopped => false,
            Self::Playing(_, _) => true,
            Self::Paused(_, _) => true,
            Self::Report(_) => true,
            Self::Succeeded => false,
        }
    }
    fn is_report(&self) -> bool {
        match self {
            Self::Report(_) => true,
            _ => false,
        }
    }
    fn close_report(&mut self, result: &MetaTestResult) {
        if result.is_passed() {
            *self = Self::Succeeded;
        } else {
            *self = Self::Stopped;
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
            Self::Playing(_, state) | Self::Paused(_, state) => {
                state.draw(context, assets, ());
            }
            Self::Report(result) => {
                result.draw(context, assets, ());
            }
            Self::Succeeded => {}
        }
    }
    fn step(&mut self, dt: f64, keyboard: &KeyboardState) -> NextScene {
        match self {
            Self::Stopped => NextScene::Continue,
            Self::Paused(_, _) => NextScene::Continue,
            Self::Playing(ref test, ref mut state) => {
                state.step(dt);
                if !state.is_complete() {
                    return NextScene::Continue;
                }

                if let Some(result) = state.result() {
                    let result = MetaTestResult::new(test.clone(), result);
                    *self = Self::Report(result);
                }
                NextScene::Continue
            }
            Self::Report(result) => {
                let result = result.clone();
                if keyboard.is_pressed("Space") || keyboard.is_pressed("Enter") {
                    self.close_report(&result);
                }
                NextScene::Continue
            }
            Self::Succeeded => NextScene::Continue,
        }
    }
}

#[derive(Clone, Debug)]
struct GodLevelRunningState {
    current_state: LevelState,
    old_state: LevelState,
    animation_time: f64,
}
impl GodLevelRunningState {
    fn new(initial_state: LevelState) -> Self {
        GodLevelRunningState {
            current_state: initial_state.clone(),
            old_state: initial_state,
            animation_time: GodLevel::MIN_SPEED,
        }
    }
    fn result(&self) -> Option<TestResult> {
        match self.current_state.success_state() {
            SuccessState::Failed => Some(TestResult::Reject),
            SuccessState::Succeeded => {
                Some(TestResult::AcceptWith(self.current_state.get_outputs()))
            }
            SuccessState::Running => None,
        }
    }

    /// is complete if all cows are in a success zone or one is in a failure zone.
    fn is_complete(&self) -> bool {
        !self.current_state.success_state().is_running() && self.animation_time > GodLevel::MIN_SPEED
    }

    fn step(&mut self, dt: f64) {
        self.animation_time += dt;
        while self.animation_time > GodLevel::MIN_SPEED && self.current_state.success_state().is_running() {
            self.animation_time -= GodLevel::MIN_SPEED;
            self.old_state.clone_from(&self.current_state);
            self.current_state.auto();
        }
    }
}
impl component::Component for GodLevelRunningState {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = util::clamp(self.animation_time / GodLevel::MIN_SPEED, 0.0, 1.0);
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
    last_press: Option<ControlButton>,
}
impl ControlPanel {
    const HALF_HEIGHT: i32 = SpriteSheet::STANDARD_HEIGHT / 2;
    const HALF_WIDTH: i32 = SpriteSheet::STANDARD_WIDTH / 2;
    const PALETTE_OFFSET: Point<i32> = Point(0, Self::HALF_HEIGHT * 3);
    const PLAY_BUTTON: CellGraphic =
        CellGraphic::new(Point(Self::HALF_WIDTH, Self::HALF_HEIGHT), Point(15, 0));
    const PAUSE_BUTTON: CellGraphic =
        CellGraphic::new(Point(Self::HALF_WIDTH * 3, Self::HALF_HEIGHT), Point(14, 0));
    const STOP_BUTTON: CellGraphic =
        CellGraphic::new(Point(Self::HALF_WIDTH * 7, Self::HALF_HEIGHT), Point(13, 0));
    const CONTROL_DIMENSIONS: component::Rect = component::Rect {
        top_left: Point(0, 0),
        dimensions: Point(Self::HALF_WIDTH * 10, Self::HALF_HEIGHT * 3),
    };

    fn new(cell_palette: CellPalette<CellType>) -> Self {
        ControlPanel {
            cell_palette: Translation::new(Self::PALETTE_OFFSET, cell_palette),
            last_press: None,
        }
    }
    fn cell_palette_value(&self) -> PaletteResult<CellType> {
        self.cell_palette.value()
    }
    fn last_press(&self) -> Option<ControlButton> {
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
            self.last_press = Some(ControlButton::Play);
            return true;
        }
        if Self::PAUSE_BUTTON.in_boundary(point) {
            self.last_press = Some(ControlButton::Pause);
            return true;
        }
        if Self::STOP_BUTTON.in_boundary(point) {
            self.last_press = Some(ControlButton::Stop);
            return true;
        }

        self.last_press = None;
        self.cell_palette.click(point)
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        Self::PLAY_BUTTON.draw(context, assets, ());
        Self::PAUSE_BUTTON.draw(context, assets, ());
        Self::STOP_BUTTON.draw(context, assets, ());

        self.cell_palette.draw(context, assets, ());
    }
}
