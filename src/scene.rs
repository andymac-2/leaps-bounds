use serde::Deserialize;

use crate::{Context2D, Assets};

use crate::component::{Component, Rect};
use crate::js_ffi::KeyboardState;
use crate::level::{overworld_level, cow_level};
use crate::point::Point;

pub enum NextScene {
    Continue,
    Return,
    Call(usize),
    Jump(usize)
}

pub trait Scene: Component {
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene;
}

pub struct Scenes {
    scenes: Vec<Box<dyn Scene<Args = ()>>>,
    current_scene: usize,
    scene_stack: Vec<usize>
}
impl Component for Scenes {
    type Args = ();
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        self.scenes[self.current_scene].draw(context, assets, ());
    }
    fn bounding_rect(&self) -> Rect {
        self.scenes[self.current_scene].bounding_rect()
    }
    fn click (&mut self, point: Point<i32>) -> bool {
        self.scenes[self.current_scene].click(point)
    }
}
impl Scene for Scenes {
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        let next_scene = self.scenes[self.current_scene].step(dt, keyboard_state);
        match next_scene {
            NextScene::Continue => NextScene::Continue,
            NextScene::Return => {
                if let Some(next_scene) = self.scene_stack.pop() {
                    self.current_scene = next_scene;
                    NextScene::Continue
                }
                else {
                    NextScene::Return
                }
            },
            NextScene::Call(next_scene) => {
                self.scene_stack.push(self.current_scene);
                self.current_scene = next_scene;
                assert!(self.current_scene < self.scenes.len());
                NextScene::Continue
            },
            NextScene::Jump(next_scene) => {
                self.current_scene = next_scene;
                assert!(self.current_scene < self.scenes.len());
                NextScene::Continue
            }
        }
    }
}

impl Scenes {
    pub fn new () -> Self {
        Scenes {
            scenes: vec![
                overworld_level(
                    include_str!("level_data/overworld_1.ron"),
                    [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2],
                ),
                Box::new(crate::level::overworld_level::OverworldLevel::default()),
                cow_level(include_str!("level_data/test_level_abcd.ron"))
            ],
            current_scene: 0,
            scene_stack: Vec::new(),
        }
    }
}

fn cow_level(string: &'static str) -> Box<dyn Scene<Args = ()>> {
    Box::new(cow_level::CowLevel::from_str(string))
}

fn overworld_level(string: &'static str, connections: [usize; 16]) -> Box<dyn Scene<Args = ()>> {
    Box::new(overworld_level::OverworldLevel::from_data(string, connections))
}