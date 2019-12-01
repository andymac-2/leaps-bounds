use crate::{Assets, Context2D};

use crate::component::{Component, NextScene, Rect, Transition, ReturnButton, Brief};
use crate::js_ffi::KeyboardState;
use crate::level::god_level::Test;
use crate::level::{cow_level, overworld_level};
use crate::point::Point;

use crate::tutorial;

pub struct Scenes {
    scenes: Vec<Box<dyn Component<DrawArgs = ()>>>,
    current_scene: usize,
    scene_stack: Vec<usize>,
}
impl Component for Scenes {
    type DrawArgs = ();
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.scenes[self.current_scene].draw(context, assets, ());
    }
    fn bounding_rect(&self) -> Rect {
        self.scenes[self.current_scene].bounding_rect()
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        self.scenes[self.current_scene].click(point)
    }
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        let next_scene = self.scenes[self.current_scene].step(dt, keyboard_state);
        match next_scene {
            NextScene::Continue => NextScene::Continue,
            NextScene::Return(object) => {
                if let Some(next_scene) = self.scene_stack.pop() {
                    self.current_scene = next_scene;
                    self.scenes[self.current_scene].returned_into(object);
                    NextScene::Continue
                } else {
                    NextScene::Return(object)
                }
            }
            NextScene::Call(next_scene, object) => {
                self.scene_stack.push(self.current_scene);
                self.current_scene = next_scene;
                self.scenes[self.current_scene].called_into(object);
                assert!(self.current_scene < self.scenes.len());
                NextScene::Continue
            }
            NextScene::Jump(next_scene, object) => {
                self.current_scene = next_scene;
                self.scenes[self.current_scene].jumped_into(object);
                assert!(self.current_scene < self.scenes.len());
                NextScene::Continue
            }
        }
    }
}

impl Scenes {
    pub fn new() -> Self {
        use crate::level::cell::Colour::*;
        use crate::level::god_level::TestTarget::*;

        let scenes = vec![
            // 0
            overworld_level(
                "overworld_0",
                include_str!("level_data/overworld_0.ron"),
                [5, 7, 3, 8, 15, 17, 19, 18, 18, 18, 18, 18, 18, 18, 18, 18],
            ),
            // 1
            cow_level(include_str!("level_data/level_0_0.ron")),
            // 2
            cow_level(include_str!("level_data/level_0_1.ron")),
            // 3
            cow_level(include_str!("level_data/level_0_2.ron")),
            //4
            god_level(
                "level_1_1",
                "ACCEPT if there is a RED\n\
                block as input, REJECT if\n\
                there is a BLUE block as\n\
                input.\n\
                Input length: 1",
                vec![
                    Test::new(vec![Red], Accept),
                    Test::new(vec![Blue], Reject),
                ]
            ),
            // 5
            tutorial(1, tutorial::LEVEL_0_0_TUTORIAL),
            // 6
            tutorial(11, tutorial::BEGINNING_TUTORIAL),
            // 7
            tutorial(2, tutorial::LEVEL_0_1_TUTORIAL),
            //8
            cow_level(include_str!("level_data/level_0_3.ron")),
            // 9
            god_level(
                "level_1_0",
                "ACCEPT all cases. (Send\n\
                all COWs to the GREEN\n\
                zone.)",
                vec![
                    Test::new(vec![], Accept),
                ]
            ),
            // 10 accept if all red
            god_level(
                "level_1_2",
                "ACCEPT if all of the\n\
                inputs are RED. REJECT if\n\
                there is a BLUE block\n\
                anywhere in the input.\n\
                Max input length: 6",
                vec![
                    Test::new(vec![Red, Red, Red, Red], Accept),
                    Test::new(vec![Red, Red, Red, Red, Red, Red], Accept),
                    Test::new(vec![], Accept),
                    Test::new(vec![Red, Red, Blue, Red], Reject),
                    Test::new(vec![Blue], Reject),
                    Test::new(vec![Blue, Blue, Blue, Blue, Blue], Reject),
                    Test::new(vec![Red, Red, Red, Red, Blue], Reject),
                ]
            ),
            // 11 main overworld
            overworld_level_no_return(
                "main_overworld",
                include_str!("level_data/main_overworld.ron"),
                [0, 20, 29, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18],
            ),
            // 12
            god_level(
                "level_2_0",
                "Return the INPUT except\n\
                swap the RED blocks with\n\
                BLUE blocks.\n\
                Max input length: 5",
                vec![
                    Test::new(vec![Red, Red, Red, Red], AcceptWith(vec![Blue, Blue, Blue, Blue])),
                    Test::new(vec![Red, Red, Red], AcceptWith(vec![Blue, Blue, Blue])),
                    Test::new(vec![], AcceptWith(vec![])),
                    Test::new(vec![Red, Red, Blue, Red], AcceptWith(vec![Blue, Blue, Red, Blue])),
                    Test::new(vec![Blue], AcceptWith(vec![Red])),
                    Test::new(vec![Blue, Blue, Blue, Blue, Blue], AcceptWith(vec![Red, Red, Red, Red, Red])),
                    Test::new(vec![Red, Blue, Red, Red, Blue], AcceptWith(vec![Blue, Red, Blue, Blue, Red])),
                ]
            ),
            // 13
            god_level(
                "level_2_1",
                "Return the input, except\n\
                remove any BLUE blocks.\n\
                Max input length: 4",
                vec![
                    Test::new(vec![Red, Red, Red, Red], AcceptWith(vec![Red, Red, Red, Red])),
                    Test::new(vec![Red, Blue, Blue, Red], AcceptWith(vec![Red, Red])),
                    Test::new(vec![], AcceptWith(vec![])),
                    Test::new(vec![Blue, Blue, Blue, Blue], AcceptWith(vec![])),
                    Test::new(vec![Blue, Red, Red, Red], AcceptWith(vec![Red, Red, Red])),
                ]
            ),
            // 14
            cow_level(include_str!("level_data/blank_level.ron")),
            // 15
            tutorial(16, tutorial::LEVEL_0_4_TUTORIAL),
            // 16
            cow_level(include_str!("level_data/level_0_4.ron")),
            // 17
            cow_level(include_str!("level_data/level_0_5.ron")),
            // 18
            tutorial(14, tutorial::INCOMPLETE_LEVEL),
            // 19
            cow_level(include_str!("level_data/level_0_6.ron")),
            // 20
            overworld_level(
                "overworld_1",
                include_str!("level_data/overworld_1.ron"),
                [21, 22, 24, 26, 23, 25, 28, 18, 18, 18, 18, 18, 18, 18, 18, 18],
            ),
            // 21
            tutorial(9, tutorial::GOD_LEVEL_TUTORIAL),
            // 22
            tutorial(4, tutorial::INPUT_TUTORIAL),
            // 23
            god_level(
                "level_1_4",
                "ACCEPT if the INPUT is\n\
                alternating RED and BLUE,\n\
                REJECT otherwise\n\
                Max input length: 8",
                vec![
                    Test::new(vec![Red, Blue, Red, Blue], Accept),
                    Test::new(vec![Blue, Red], Accept),
                    Test::new(vec![Red, Blue, Red, Blue, Red, Blue, Red, Blue, Red], Accept),
                    Test::new(vec![Red, Blue, Blue, Blue], Reject),
                    Test::new(vec![], Accept),
                    Test::new(vec![Red, Red], Reject),
                ]
            ),
            // 24
            tutorial(10, tutorial::SPEED_TUTORIAL),
            // 25
            god_level(
                "level_1_5",
                "COPY the INPUT to the\n\
                OUTPUT. The INPUT may\n\
                contain RED or BLUE blocks\n\
                Max input length: 8",
                vec![
                    Test::new(vec![Red, Blue, Red, Blue], AcceptWith(vec![Red, Blue, Red, Blue])),
                    Test::new(vec![Red], AcceptWith(vec![Red])),
                    Test::new(vec![], AcceptWith(vec![])),
                    Test::new(vec![Red, Red, Blue, Blue, Red, Blue, Red, Blue], AcceptWith(vec![Red, Red, Blue, Blue, Red, Blue, Red, Blue])),
                    Test::new(vec![Red, Red, Red, Red, Red, Red, Red, Red], AcceptWith(vec![Red, Red, Red, Red, Red, Red, Red, Red])),
                ]
            ),
            // 26
            tutorial(27, tutorial::OUTPUT_TUTORIAL),
            // 27
            god_level(
                "level_1_3",
                "Write a RED block to the\n\
                OUTPUT, then ACCEPT.\n\
                Input length: 1",
                vec![
                    Test::new(vec![], AcceptWith(vec!(Red))),
                ]
            ),
            // 28
            god_level(
                "level_1_6",
                "COPY the INPUT to the\n\
                OUTPUT. The INPUT may\n\
                contain RED, BLUE or\n\
                GREEN blocks\n\
                Max input length: 8",
                vec![
                    Test::new(vec![Red, Blue, Red, Green], AcceptWith(vec![Red, Blue, Red, Green])),
                    Test::new(vec![Green], AcceptWith(vec![Green])),
                    Test::new(vec![], AcceptWith(vec![])),
                    Test::new(vec![Red, Green, Blue, Blue, Green, Blue, Red, Green], AcceptWith(vec![Red, Green, Blue, Blue, Green, Blue, Red, Green])),
                    Test::new(vec![Red, Red, Red, Red, Green, Red, Red, Red], AcceptWith(vec![Red, Red, Red, Red, Green, Red, Red, Red])),
                ]
            ),
            // 29
            overworld_level(
                "overworld_2",
                include_str!("level_data/overworld_2.ron"),
                [12, 13, 30, 31, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18, 18],
            ),
            // 30
            god_level(
                "level_1_3",
                "Accept if there are more\n\
                RED blocks than BLUE blocks\n\
                in the INPUT.\n\
                Max length: 8",
                vec![
                    Test::new(vec![Red, Blue, Red, Blue, Blue, Blue, Red, Blue], Reject),
                    Test::new(vec![Red, Blue, Red, Red, Red, Blue, Red, Red], Accept),
                    Test::new(vec![Blue, Blue, Red, Blue, Red, Blue, Red, Red], Reject),
                    Test::new(vec![Blue, Red, Red, Blue, Red, Blue, Red, Red], Accept),
                    Test::new(vec![], Reject),
                    Test::new(vec![Red], Accept),
                ]
            ),
            // 31
            god_level(
                "level_1_3",
                "Return the INPUT but with\n\
                all of the RED inputs at\n\
                the start, and all of the\n\
                BLUE inputs at the end\n\
                Max length: 8",
                vec![
                    Test::new(vec![Red, Blue, Red, Red, Blue, Blue, Red, Blue], AcceptWith(vec![Red, Red, Red, Red, Blue, Blue, Blue, Blue])),
                    Test::new(vec![Red, Blue, Red, Red, Red, Blue, Red, Red], AcceptWith(vec![Red, Red, Red, Red, Red, Red, Blue, Blue])),
                    Test::new(vec![Blue, Blue, Red, Blue, Red, Blue, Red, Red], AcceptWith(vec![Red, Red, Red, Red, Blue, Blue, Blue, Blue])),
                    Test::new(vec![Blue, Red, Red, Blue, Red, Blue, Red, Red], AcceptWith(vec![Red, Red, Red, Red, Red, Blue, Blue, Blue])),
                    Test::new(vec![], AcceptWith(vec![])),
                    Test::new(vec![Blue, Red],AcceptWith(vec![Red, Blue])),
                ]
            ),
        ];

        // MAX BRIEF COLUMN WIDTH: 44
        Scenes {
            scenes,
            current_scene: 6,
            scene_stack: Vec::new(),
        }
    }
}

fn cow_level(string: &'static str) -> Box<dyn Component<DrawArgs = ()>> {
    let level = cow_level::CowLevel::from_str(string);
    Box::new(Transition::new(ReturnButton::new(level)))
}

fn overworld_level_no_return(
    name: &'static str,
    string: &'static str,
    connections: [usize; 16],
) -> Box<dyn Component<DrawArgs = ()>> {
    let level = overworld_level::OverworldLevel::from_data(name, string, connections);
    Box::new(Transition::new(level))
}
fn overworld_level(
    name: &'static str,
    string: &'static str,
    connections: [usize; 16],
) -> Box<dyn Component<DrawArgs = ()>> {
    let level = overworld_level::OverworldLevel::from_data(name, string, connections);
    Box::new(Transition::new(ReturnButton::new(level)))
}

fn god_level(name: &'static str, description: &'static str, tests: Vec<Test>) -> Box<dyn Component<DrawArgs = ()>> {
    let level = crate::level::god_level::GodLevel::new(name, tests);
    Box::new(Transition::new(Brief::new(description, ReturnButton::new(level))))
}

fn tutorial(
    destination: usize,
    screens: &'static [tutorial::Screen],
) -> Box<dyn Component<DrawArgs = ()>> {
    Box::new(Transition::new(tutorial::Tutorial::new(destination, screens)))
}
