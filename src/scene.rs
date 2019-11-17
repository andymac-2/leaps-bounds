use std::collections::HashMap;

use serde::Deserialize;

use crate::{Context2D, Assets};

use crate::component::{Component, Rect};
use crate::js_ffi::KeyboardState;
use crate::level::{overworld_level, cow_level};
use crate::point::Point;

// A generic data object, kind of like JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Object {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Array(Vec<Object>),
    Map(HashMap::<String, Object>),
}

pub enum NextScene {
    Continue,
    Return(Object),
    Call(usize, Object),
    Jump(usize, Object)
}

pub trait Scene: Component {
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene;

    fn returned_into(&mut self, _object: Object) {}
    fn called_into(&mut self, _object: Object) {}
    fn jumped_into(&mut self, object: Object) {
        self.called_into(object)
    }
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
            NextScene::Return(object) => {
                if let Some(next_scene) = self.scene_stack.pop() {
                    self.current_scene = next_scene;
                    self.scenes[self.current_scene].returned_into(object);
                    NextScene::Continue
                }
                else {
                    NextScene::Return(object)
                }
            },
            NextScene::Call(next_scene, object) => {
                self.scene_stack.push(self.current_scene);
                self.current_scene = next_scene;
                self.scenes[self.current_scene].called_into(object);
                assert!(self.current_scene < self.scenes.len());
                NextScene::Continue
            },
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
    pub fn new () -> Self {
        Scenes {
            scenes: vec![
                overworld_level(
                    include_str!("level_data/overworld_1.ron"),
                    [1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2, 1, 2],
                ),
                cow_level(include_str!("level_data/level_0_0.ron")),
                cow_level(include_str!("level_data/level_0_1.ron")),
                cow_level(include_str!("level_data/test_level_abcd.ron")),
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