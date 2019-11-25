use crate::{Assets, Context2D};

use crate::component::{Component, NextScene, Rect, Transition, ReturnButton};
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

        Scenes {
            scenes: vec![
                // 0
                overworld_level(
                    include_str!("level_data/overworld_1.ron"),
                    [5, 7, 3, 8, 4, 9, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50],
                ),
                // 1
                cow_level(include_str!("level_data/level_0_0.ron")),
                // 2
                cow_level(include_str!("level_data/level_0_1.ron")),
                // 3
                cow_level(include_str!("level_data/level_0_2.ron")),
                //4
                god_level(vec![
                    Test::new(vec![Red], Accept),
                    Test::new(vec![Blue], Reject),
                ]),
                // 5
                tutorial(1, tutorial::LEVEL_0_TUTORIAL),
                // 6
                tutorial(11, tutorial::BEGINNING_TUTORIAL),
                // 7
                tutorial(2, tutorial::LEVEL_0_1_TUTORIAL),
                //8
                cow_level(include_str!("level_data/level_0_3.ron")),
                // 9 test god level
                god_level(vec![
                    Test::new(vec![], Accept),
                ]),
                // 10 
                god_level(vec![
                    Test::new(vec![Red, Red, Red, Red], Accept),
                    Test::new(vec![Red, Red, Red, Red, Red, Red], Accept),
                    Test::new(vec![], Accept),
                    Test::new(vec![Red, Red, Blue, Red], Reject),
                    Test::new(vec![Blue], Reject),
                    Test::new(vec![Blue, Blue, Blue, Blue, Blue], Reject),
                    Test::new(vec![Red, Red, Red, Red, Blue], Reject),
                ]),
                // 11 main overworld
                overworld_level_no_return(
                    include_str!("level_data/main_overworld.ron"),
                    [0, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50],
                ),
            ],
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
    string: &'static str,
    connections: [usize; 16],
) -> Box<dyn Component<DrawArgs = ()>> {
    let level = overworld_level::OverworldLevel::from_data(string, connections);
    Box::new(Transition::new(level))
}
fn overworld_level(
    string: &'static str,
    connections: [usize; 16],
) -> Box<dyn Component<DrawArgs = ()>> {
    let level = overworld_level::OverworldLevel::from_data(string, connections);
    Box::new(Transition::new(ReturnButton::new(level)))
}

fn god_level(tests: Vec<Test>) -> Box<dyn Component<DrawArgs = ()>> {
    let level = crate::level::god_level::GodLevel::new(tests);
    Box::new(Transition::new(ReturnButton::new(level)))
}

fn tutorial(
    destination: usize,
    screens: &'static [tutorial::Screen],
) -> Box<dyn Component<DrawArgs = ()>> {
    Box::new(Transition::new(tutorial::Tutorial::new(destination, screens)))
}
