use std::convert::TryFrom;

use crate::component::Rect;
use crate::component::{NextScene, Object, Translation};
use crate::point::Point;
use crate::util;
use crate::{component, Assets, Context2D, KeyboardState, SpriteSheet};

use super::cell::{cell_cursor, CellGraphic, CellPalette, CellType, Colour, PaletteResult};
use super::cow_level::CowLevel;
use super::{LevelState, SuccessState};

#[derive(Clone, Debug)]
pub struct Test {
    input: Vec<Colour>,
    output: TestTarget,
}
impl Test {
    pub fn new(input: Vec<Colour>, output: TestTarget) -> Test {
        Test { input, output }
    }
}
#[derive(Clone, Debug)]
pub enum TestTarget {
    Reject,
    Accept,
    AcceptWith(Vec<Colour>),
}

#[derive(Clone, Debug)]
pub enum TestResult {
    Reject,
    AcceptWith(Vec<Colour>),
    NotEnoughInputSpace,
}

#[derive(Clone, Debug)]
pub struct MetaTestResult {
    test: Test,
    result: TestResult,
}
impl<'a> MetaTestResult {
    fn new(test: Test, result: TestResult) -> Self {
        MetaTestResult { test, result }
    }
    fn is_passed(&self) -> bool {
        match (&self.test.output, &self.result) {
            (TestTarget::Reject, TestResult::Reject) => true,
            (TestTarget::Accept, TestResult::AcceptWith(_)) => true,
            (TestTarget::AcceptWith(ideal), TestResult::AcceptWith(real)) => ideal == real,
            (_, _) => false,
        }
    }
    fn draw_colours(context: &Context2D, assets: &Assets, colours: &[Colour], offset: Point<f64>) {
        if colours.is_empty() {
            context.save();

            context.set_font("10px KongText");
            context.set_text_align("center");
            let black = wasm_bindgen::JsValue::from_str("black");
            context.set_fill_style(&black);
            context.fill_text("<empty>", offset.x(), offset.y()).unwrap();

            context.restore();
            return;
        }

        let width: f64 = f64::from(SpriteSheet::STANDARD_WIDTH) * colours.len() as f64;
        let left: f64 = offset.x() - width / 2.0;
        
        for (index, colour) in colours.iter().enumerate() {
            let cursor: f64 = f64::from(SpriteSheet::STANDARD_WIDTH) * index as f64;
            let x = left + cursor;
            assets.blocks.draw(context, Point(*colour as u8, 0), Point(x, offset.y()));
        }
    }

    const REPORT_BG: Rect = Rect::indexed(Point(1, 0), Rect::FOUR_BY_TWO);
    const BOUNDING_RECT: Rect = CowLevel::BOUNDING_RECT;
    const CENTRE: f64 =
        (Self::BOUNDING_RECT.top_left.0 + (Self::BOUNDING_RECT.dimensions.0 / 2)) as f64;
    const LEFT_COLUMN: f64 = Self::CENTRE * 0.5;
    const RIGHT_COLUMN: f64 = Self::CENTRE * 1.5;

    const TOP_MARGIN: f64 = 60.0;
    const RESULT_TOP: f64 = 90.0;
    const INPUT_TOP: f64 = 110.0;
    const SUBHEADING_TOP: f64 = 150.0;
}
impl component::Component for MetaTestResult {
    type DrawArgs = ();
    fn bounding_rect(&self) -> Rect {
        Self::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        assets
            .misc
            .draw_with_rect(context, &Self::REPORT_BG, &Self::BOUNDING_RECT);

        util::with_saved_context(context, || {
            context.set_font("25px KongText");
            context.set_text_align("center");
            let black = wasm_bindgen::JsValue::from_str("black");
            let green = wasm_bindgen::JsValue::from_str("#47a624");
            let red = wasm_bindgen::JsValue::from_str("#bb0015");
            
            context.set_fill_style(&black);
            context
                .fill_text("Report:", Self::CENTRE, Self::TOP_MARGIN)
                .unwrap();

            let (colour, text) = if self.is_passed() {
                (&green, "Pass!")
            }
            else {
                (&red, "Fail!")
            };
            context.set_fill_style(colour);
            context
                .fill_text(text, Self::CENTRE, Self::RESULT_TOP)
                .unwrap();

            context.set_font("15px KongText");
            context.set_fill_style(&black);
            context
                .fill_text("Input:", Self::CENTRE, Self::INPUT_TOP)
                .unwrap();
            context
                .fill_text("Expected:", Self::LEFT_COLUMN, Self::SUBHEADING_TOP)
                .unwrap();
            context
                .fill_text("Found:", Self::RIGHT_COLUMN, Self::SUBHEADING_TOP)
                .unwrap();
            
            Self::draw_colours(context, assets, &self.test.input, Point(Self::CENTRE, Self::INPUT_TOP + 3.0));

            match &self.test.output{
                TestTarget::Reject => {
                    context.set_fill_style(&red);
                    context
                        .fill_text("Reject", Self::LEFT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                },
                TestTarget::Accept => {
                    context.set_fill_style(&green);
                    context
                        .fill_text("Accept", Self::LEFT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                },
                TestTarget::AcceptWith(ideal) => {
                    context.set_fill_style(&green);
                    context
                        .fill_text("Accept", Self::LEFT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                    Self::draw_colours(context, assets, ideal, Point(Self::LEFT_COLUMN, Self::SUBHEADING_TOP + 23.0));
                },
            }

            match &self.result {
                TestResult::Reject => {
                    context.set_fill_style(&red);
                    context
                        .fill_text("Reject", Self::RIGHT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                },
                TestResult::AcceptWith(result) => {
                    context.set_fill_style(&green);
                    context
                        .fill_text("Accept", Self::RIGHT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                    Self::draw_colours(context, assets, result, Point(Self::RIGHT_COLUMN, Self::SUBHEADING_TOP + 23.0));
                }
                TestResult::NotEnoughInputSpace => {
                    context.set_fill_style(&black);
                    context
                        .fill_text("Not enough", Self::RIGHT_COLUMN, Self::SUBHEADING_TOP + 20.0)
                        .unwrap();
                    context
                        .fill_text("room.", Self::RIGHT_COLUMN, Self::SUBHEADING_TOP + 40.0)
                        .unwrap();
                }
            }
        });
    }
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
        let state = self.initial_state.clone();
        let test = self.get_current_test().clone();

        self.running_state.stop();
        self.running_state.start(state, test, self.speed);

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
            }
            ControlButton::Stop => {
                self.current_test = 0;
                self.running_state.stop()
            }
            ControlButton::Pause => self.running_state.pause(),
        }
    }
}
impl component::Component for GodLevel {
    type DrawArgs = ();
    fn called_into(&mut self, _object: Object) {
        self.current_test = 0;
        self.running_state.stop();
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

        if !self.running_state.is_stopped() {
            return false;
        }

        let value = self.control_panel.cell_palette_value();
        self.initial_state.set_cell_at_point(point, value);
        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.fill_bg(context, super::BG_FILL);

        if self.running_state.is_drawable() {
            self.initial_state
                .draw(context, assets, &self.initial_state, 0.0);
        } else {
            self.running_state.draw(context, assets, ());
        }

        if !self.running_state.is_report() {
            self.control_panel.fill_bg(context, cell_cursor::BG_COLOUR);
            self.control_panel.draw(context, assets, ());
        }
    }
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        self.running_state.step(dt, keyboard_state);
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
    Paused(Test, GodLevelRunningState),
    Playing(Test, GodLevelRunningState),
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
    fn start(&mut self, mut state: LevelState, test: Test, speed: f64) {
        assert!(self.is_stopped());
        if let Ok(()) = state.set_inputs(&test.input) {
            *self = Self::Playing(test, GodLevelRunningState::new(state, speed));
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
                if keyboard.is_pressed("Space") || keyboard.is_pressed("Enter") {
                    if result.is_passed() {
                        *self = Self::Succeeded;
                    } else {
                        *self = Self::Stopped;
                    }
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
        let anim_progress = util::clamp(self.animation_time / self.speed, 0.0, 1.0);
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
