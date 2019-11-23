use crate::component::{NextScene, Object, Rect};
use crate::point::Point;
use crate::sprite_sheet::SpriteSheet;
use crate::{component, Assets, Context2D, KeyboardState};

#[derive(Clone, Debug)]
pub struct Tutorial {
    cursor: usize,
    text_cursor: usize,
    animation_time: f64,
    screens: &'static [Screen],
    destination: usize,
}
impl Tutorial {
    pub const fn new(destination: usize, screens: &'static [Screen]) -> Self {
        Tutorial {
            cursor: 0,
            text_cursor: 0,
            animation_time: 0.0,
            screens,
            destination,
        }
    }
    fn next_screen(&mut self) {
        self.cursor += 1;
        self.text_cursor = 0;
    }
    fn current_text(&self) -> &'static str {
        self.screens[self.cursor].text
    }
    fn current_image(&self) -> &Rect {
        &self.screens[self.cursor].image
    }
    fn current_icon(&self) -> &Rect {
        &self.screens[self.cursor].icon
    }

    fn is_screen_finished(&self) -> bool {
        self.text_cursor >= self.current_text().len()
    }
    fn finish_screen(&mut self) {
        self.text_cursor = usize::max_value();
    }

    fn reset(&mut self) {
        self.cursor = 0;
        self.text_cursor = 0;
        self.animation_time = 0.0;
    }

    const TEXT_SPEED: f64 = 30.0;

    const BG_IMG_RECT: Rect = Rect::new(Point(0, 0), Point(64, 32));
    const BOUNDING_RECT: Rect = crate::level::cow_level::CowLevel::BOUNDING_RECT;

    const IMG_DIMS: Point<i32> = Point(115, 115);

    const LL_CORNER: Rect = Rect::new(
        Point(0, Self::BOUNDING_RECT.dimensions.1 / 2 + 10),
        Self::IMG_DIMS,
    );

    const IMG_HEIGHT: i32 = Self::IMG_DIMS.1;
    const IMG_CENTRE: Point<i32> = Point(
        Self::BOUNDING_RECT.dimensions.0 * 2 / 3,
        (Self::BOUNDING_RECT.dimensions.1 * 2 / 3) + 20,
    );

    const LINE_HEIGHT: f64 = 14.0;
    const LEFT_MARGIN: f64 = 30.0;
    const TOP_MARGIN: f64 = 40.0;
    // press spacebar text left margin
    const RIGHT_TEXT: f64 = 340.0;
    // press spacebar text top margin
    const BOTTOM_TEXT: f64 = Self::LINE_HEIGHT * 5.0 + Self::TOP_MARGIN;
}
impl component::Component for Tutorial {
    type DrawArgs = ();
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        self.animation_time += dt;
        if self.animation_time > Self::TEXT_SPEED {
            self.animation_time = 0.0;
            self.text_cursor = self.text_cursor.saturating_add(1);
        }

        if keyboard_state.is_pressed("Space") {
            if self.is_screen_finished() {
                self.next_screen();
            } else {
                self.finish_screen()
            }
        };

        if self.cursor >= self.screens.len() {
            self.reset();
            NextScene::Jump(self.destination, Object::Null)
        } else {
            NextScene::Continue
        }
    }
    fn bounding_rect(&self) -> component::Rect {
        Self::BOUNDING_RECT
    }
    fn click(&mut self, _point: Point<i32>) -> bool {
        self.next_screen();
        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: Self::DrawArgs) {
        assets
            .misc
            .draw_with_rect(context, &Self::BG_IMG_RECT, &Self::BOUNDING_RECT);
        assets
            .misc
            .draw_with_rect(context, self.current_icon(), &Self::LL_CORNER);
        assets.misc.draw_with_source_height(
            context,
            self.current_image(),
            Self::IMG_CENTRE,
            Self::IMG_HEIGHT,
        );

        context.set_font("11px KongText");
        let black = wasm_bindgen::JsValue::from_str("black");
        context.set_fill_style(&black);

        let mut baseline = Self::TOP_MARGIN;
        let mut chars_left_to_print = self.text_cursor;

        for line in self.current_text().lines() {
            if chars_left_to_print == 0 {
                break;
            }

            if chars_left_to_print >= line.len() {
                context
                    .fill_text(line, Self::LEFT_MARGIN, baseline)
                    .unwrap();
                chars_left_to_print -= line.len();
            } else {
                context
                    .fill_text(&line[(0..chars_left_to_print)], Self::LEFT_MARGIN, baseline)
                    .unwrap();
                chars_left_to_print = 0;
            };

            baseline += Self::LINE_HEIGHT;
        }

        if self.is_screen_finished() {
            context
                .fill_text("press SPACE", Self::RIGHT_TEXT, Self::BOTTOM_TEXT)
                .unwrap();
        }
    }
}

#[derive(Clone, Debug)]
pub struct Screen {
    icon: Rect,
    text: &'static str,
    image: Rect,
}
impl Screen {
    const fn new(icon: Rect, image: Rect, text: &'static str) -> Self {
        Screen { icon, text, image }
    }
}

const fn indexed_rect(index: Point<u8>, dimensions: Point<i32>) -> Rect {
    Rect::new(
        Point(index.0 as i32 * dimensions.0, index.1 as i32 * dimensions.1),
        dimensions,
    )
}

const ONE_BY_ONE: Point<i32> = Point(SpriteSheet::STANDARD_WIDTH, SpriteSheet::STANDARD_HEIGHT);
const TWO_BY_TWO: Point<i32> = Point(
    SpriteSheet::STANDARD_WIDTH * 2,
    SpriteSheet::STANDARD_HEIGHT * 2,
);
const FOUR_BY_FOUR: Point<i32> = Point(
    SpriteSheet::STANDARD_WIDTH * 4,
    SpriteSheet::STANDARD_HEIGHT * 4,
);
const FOUR_BY_TWO: Point<i32> = Point(
    SpriteSheet::STANDARD_WIDTH * 4,
    SpriteSheet::STANDARD_HEIGHT * 2,
);

const NORMAL_ICON: Rect = indexed_rect(Point(0, 2), ONE_BY_ONE);
const EXCITED_ICON: Rect = indexed_rect(Point(1, 2), ONE_BY_ONE);
const PHEW_ICON: Rect = indexed_rect(Point(2, 2), ONE_BY_ONE);
const INDICATE_ICON: Rect = indexed_rect(Point(3, 2), ONE_BY_ONE);
const JUMP_ICON: Rect = indexed_rect(Point(4, 2), ONE_BY_ONE);
const HMM_ICON: Rect = indexed_rect(Point(5, 2), ONE_BY_ONE);
const INDICATE2_ICON: Rect = indexed_rect(Point(6, 2), ONE_BY_ONE);

const NO_IMG: Rect = indexed_rect(Point(6, 1), ONE_BY_ONE);
const COW_IMG: Rect = indexed_rect(Point(0, 2), TWO_BY_TWO);
const BROWN_COW_IMG: Rect = indexed_rect(Point(1, 2), TWO_BY_TWO);
const TIED_COW_IMG: Rect = indexed_rect(Point(1, 2), FOUR_BY_TWO);
const RED_GREEN_IMG: Rect = indexed_rect(Point(2, 0), FOUR_BY_TWO);
const GOD_LEVEL_IMG: Rect = indexed_rect(Point(0, 3), TWO_BY_TWO);

#[rustfmt::skip]
pub const BEGINNING_TUTORIAL: &[Screen] = &[
    Screen::new(PHEW_ICON, NO_IMG,
"Hello there! My name is Gloop and I'm
here to show you around. Use the ARROW
KEYS to move and the SPACEBAR to do
pretty much anything else",
    ),
    Screen::new(NORMAL_ICON, NO_IMG,
"I'll be waiting for you in level 0, so
follow me there! Press SPACE to enter the
level.",
    ),
];

#[rustfmt::skip]
pub const LEVEL_0_TUTORIAL: &[Screen] = &[
    Screen::new(NORMAL_ICON, COW_IMG,
"Helcome to the Logically Executed
Automatic Pasture! We have discovered
that COWs are happier when they're given
something to do.",
    ),
    Screen::new(PHEW_ICON, COW_IMG,
"Oh, did you think I said cow?
I Actually meant COW. That stands for
Carry On Walking. Our COW's will always 
carry on walking. I guarantee it.",
    ),
    Screen::new(HMM_ICON, COW_IMG,
"Well I suppose all of our COWs happen
to be cows as well. Nothing can be done
about that I'm afraid.",
    ),
    Screen::new(NORMAL_ICON, COW_IMG, 
"It's hard to say Logically Executed
Automatic Pasture. I like to say LEAP for
short. It's your job to make sure all of
our COWs do exactly what they are told
whilst in their pasture."
    ),
    Screen::new(NORMAL_ICON, COW_IMG, 
"To make sure all of our COWs behave
correctly in the LEAP, we have used
BOWNDs"
    ),
    Screen::new(NORMAL_ICON, TIED_COW_IMG, 
"BOWNDS are Bovine OWNership Devices. We
use them to allow one COW to control
another."
    ),
    Screen::new(PHEW_ICON, TIED_COW_IMG,
"Whoops, did I say BOWNDs?
I actually meant bounds. We just tie our
COWs together with rope. It's very high
tech, and three times cheaper.",
    ),
    Screen::new(INDICATE_ICON, RED_GREEN_IMG,
"Your aim is to get every COW into the
GREEN area without letting any COW reach
the RED area.",
    ),
    Screen::new(NORMAL_ICON, RED_GREEN_IMG,
"To do that use the arrow keys to move
and the SPACE bar to wait.",
    ),
    Screen::new(NORMAL_ICON, RED_GREEN_IMG,
"To see any of these tutorials again, you
can re-enter the level after you have
completed it.",
    ),
    Screen::new(NORMAL_ICON, RED_GREEN_IMG,
"Allright, That's enough training for now.
See if you can get some COWs walking!",
    ),
];


const ARROWS_IMG: Rect = indexed_rect(Point(2, 1), FOUR_BY_TWO);

#[rustfmt::skip]
pub const LEVEL_0_1_TUTORIAL: &[Screen] = &[
    Screen::new(NORMAL_ICON, BROWN_COW_IMG,
"If a COW is not bound to another, then
it is said to be \"Not OWned\" or NOW for
short",
    ),
Screen::new(EXCITED_ICON, BROWN_COW_IMG,
"If a COW can OWN another COW it is said
to \"Have OWnership\" or HOW for short.",
    ),
    Screen::new(HMM_ICON, BROWN_COW_IMG,
"Brown cows are both not owned, and can
have ownership of other cows. In other
words: HOW NOW brown COW.",
    ),
    Screen::new(NORMAL_ICON, BROWN_COW_IMG,
"A COW that is not owned by any other cow
will Carry On Walking forever.",
    ),
    Screen::new(HMM_ICON, ARROWS_IMG,
"These COWs will always obey signs written
on the floor. For example, they will
always follow the direction of an arrow
on the ground.",
    ),
    Screen::new(INDICATE_ICON, RED_GREEN_IMG,
"Remember to make sure that all COWs end
up in the GREEN areas and not the RED
ones.",
    ),
];

#[rustfmt::skip]
pub const GOD_LEVEL_TUTORIAL: &[Screen] = &[
    Screen::new(NORMAL_ICON, NO_IMG,
"Now that you know how all of the blocks
work, it's time to give you free reign
over the design over the LEAP.",
    ),
    Screen::new(INDICATE_ICON, GOD_LEVEL_IMG,
"Levels whewre you have free reign will
be coloured BROWN on you map.",
    ),
    Screen::new(NORMAL_ICON, NO_IMG,
"Once you have designed the LEAP, the
COW's will be complely automatic. You
won't have any control until they are
finished walking.",
    ),
    Screen::new(INDICATE2_ICON, RED_GREEN_IMG,
"The COW's will finish walking when
either all of them are in a GREEN zone,
or at least one COW is in a RED zone",
    ),
Screen::new(NORMAL_ICON, NO_IMG,
"For the next level, make sure that all
COWs end up in a GREEN zone."
    ),
    Screen::new(NORMAL_ICON, NO_IMG,
"Once you have finished designing your
LEAP, press the play button to test that
everything works."
    ),
];
