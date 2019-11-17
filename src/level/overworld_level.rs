use serde::{Serialize, Deserialize};

use crate::{component, KeyboardState, Context2D, Assets, scene, util};
use crate::scene::NextScene;
use crate::point::Point;

use super::{cell, board, LevelState, Level, KeyboardCommand};
use super::cell::{OverworldCell, OverworldCellType};
use super::cow::Cow;
use super::cow_level::CowLevel;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OverworldLevelState {
    board: board::LevelLayer<cell::OverworldCell>,
    player: Cow,
    animation_frame: u8,
}
impl Default for OverworldLevelState {
    fn default() -> Self {
        OverworldLevelState {
            board: board::LevelLayer::default(),
            player: Cow::default(),
            animation_frame: 0,
        }
    }
}
impl component::Component for OverworldLevelState {
    type Args = (Point<i32>, f64);
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(&self, context: &Context2D, assets: &Assets, (old_position, anim_progress): Self::Args) {
        self.board.draw(
            context, 
            &assets.blocks, 
            Point(0, 0),
            Point(CowLevel::LEVEL_WIDTH, CowLevel::LEVEL_HEIGHT),
        );

        self.player.draw(
            context, 
            &assets.sprites,
            old_position,
            anim_progress,
            self.animation_frame
        );
    }
}
impl OverworldLevelState {
    fn get_player_position(&self) -> Point<i32> {
        self.player.get_position()
    }
    fn get_current_cell(&self) -> &cell::OverworldCell {
        let position = self.get_player_position();
        self.board.get_cell(&position)
    }
    fn set_cell(&mut self, point: Point<i32>, value: cell::OverworldCell) {
        let index = board::get_grid_index(point);
        self.board.set_cell(index, value);
    }
    fn command(&mut self, command: KeyboardCommand) {
        self.animation_frame = (self.animation_frame + 1) % LevelState::TOTAL_ANIMATION_FRAMES;
        match command {
            KeyboardCommand::Direction(direction) => 
                self.player.walk_stop(&self.board, direction),
            KeyboardCommand::Space => {},
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverworldLevel {
    cell_palette: cell::CellPalette<cell::OverworldCellType>,
    state: OverworldLevelState,
    old_position: Point<i32>,
    animation_time: f64,
    levels: [usize; 16],
}
impl Default for OverworldLevel {
    fn default() -> Self {
        let state = OverworldLevelState::default();
        let old_position = state.get_player_position();

        OverworldLevel {
            cell_palette: cell::CellPalette::new(OverworldCellType::full_palette()),
            state,
            old_position,
            animation_time: 0.0,
            levels: [usize::max_value(); 16],
        }
    }
}
impl Level for OverworldLevel {
    fn is_finished_animating(&self) -> bool {
        self.animation_time > CowLevel::ANIMATION_TIME + CowLevel::COOLDOWN_TIME
    }
}
impl component::Component for OverworldLevel {
    type Args = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if self.cell_palette.click(point) {
            return true;
        }

        let value: cell::OverworldCell = self.cell_palette.value().into();
        self.state.set_cell(point, value.clone());

        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = util::clamp(self.animation_time / CowLevel::ANIMATION_TIME, 0.0, 1.0);
        self.fill_bg(context, "rgb(113, 46, 25)");

        self.state.draw(context, assets, (self.old_position, anim_progress));
        self.cell_palette.draw(context, assets, ());
    }
}
impl scene::Scene for OverworldLevel {
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        self.animation_time += dt;

        // enter and exot into levels
        // let success_state = self.success_state();
        // if let SuccessState::Succeeded = success_state {
        //     if self.is_finished_animating() {
        //         crate::console_log!("Success!");
        //         return NextScene::Return;
        //     }
        //     return NextScene::Continue;
        // }

        if keyboard_state.is_pressed("KeyL") {
            self.log_level()
        }

        if let Some(command) = self.get_keyboard_command(keyboard_state) {
            self.old_position = self.state.get_player_position();
            if command.is_space() {
                if let OverworldCell::Level(id, _) = self.current_cell() {
                    let next_level = self.levels[usize::from(*id)];
                    return NextScene::Call(next_level);
                }
            }
            self.state.command(command);

            self.animation_time = 0.0;
        };

        return NextScene::Continue;
    }
}
impl OverworldLevel {
    fn log_level(&self) {
        crate::console_log!("{}", ron::ser::to_string(&self.state).unwrap());
    }
    pub fn from_data(string: &str, connections: [usize; 16]) -> Self {
        let state: OverworldLevelState = ron::de::from_str(string).unwrap();
        let position = state.get_player_position();

        OverworldLevel {
            cell_palette: cell::CellPalette::new(OverworldCellType::full_palette()),
            state,
            old_position: position,
            animation_time: 0.0,
            levels: connections,
        }
    }
    fn current_cell(&self) -> &cell::OverworldCell {
        self.state.get_current_cell()
    }
}