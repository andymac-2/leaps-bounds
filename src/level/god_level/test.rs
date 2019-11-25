use crate::{Context2D, Assets, SpriteSheet, Point, util};
use crate::component::{Component, Rect};
use crate::level::cell::{Colour};

#[derive(Clone, Debug)]
pub struct Test {
    input: Vec<Colour>,
    output: TestTarget,
}
impl Test {
    pub fn new(input: Vec<Colour>, output: TestTarget) -> Test {
        Test { input, output }
    }
    pub fn input(&self) -> &[Colour] {
        &self.input
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
    pub fn new(test: Test, result: TestResult) -> Self {
        MetaTestResult { test, result }
    }
    pub fn is_passed(&self) -> bool {
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
            context.fill_text("<empty>", offset.x(), offset.y() + 15.0).unwrap();

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
    const BOUNDING_RECT: Rect = crate::level::cow_level::CowLevel::BOUNDING_RECT;
    const CENTRE: f64 =
        (Self::BOUNDING_RECT.top_left.0 + (Self::BOUNDING_RECT.dimensions.0 / 2)) as f64;
    const LEFT_COLUMN: f64 = Self::CENTRE * 0.5;
    const RIGHT_COLUMN: f64 = Self::CENTRE * 1.5;

    const TOP_MARGIN: f64 = 60.0;
    const RESULT_TOP: f64 = 90.0;
    const INPUT_TOP: f64 = 110.0;
    const SUBHEADING_TOP: f64 = 150.0;
}
impl Component for MetaTestResult {
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