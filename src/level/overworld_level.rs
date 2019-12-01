use serde::{Deserialize, Serialize};

use crate::component::{NextScene, Object};
use crate::direction::Direction;
use crate::point::Point;
use crate::{component, util, Assets, Context2D, KeyboardState};

use super::cell::{cell_cursor, OverworldCell, OverworldCellType, Surroundings};
use super::cow::Cow;
use super::cow_level::CowLevel;
use super::{board, cell, KeyboardCommand, Level, LevelState};

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
    type DrawArgs = (Point<i32>, f64);
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn draw(
        &self,
        context: &Context2D,
        assets: &Assets,
        (old_position, anim_progress): Self::DrawArgs,
    ) {
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
            self.animation_frame,
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
    fn get_cell(&self, point: &Point<i32>) -> &cell::OverworldCell {
        self.board.get_cell(point)
    }
    fn set_cell_at_cursor(&mut self, point: Point<i32>, value: cell::OverworldCell) {
        let index = board::get_grid_index(point);
        self.board.set_cell(index, value);
    }
    fn set_cell_at_index(&mut self, index: Point<i32>, value: cell::OverworldCell) {
        self.board.set_cell(index, value);
    }
    fn command(&mut self, command: KeyboardCommand) {
        self.animation_frame = (self.animation_frame + 1) % LevelState::TOTAL_ANIMATION_FRAMES;
        match command {
            KeyboardCommand::Direction(direction) => self.player.walk_stop(&self.board, direction),
            KeyboardCommand::Space => {}
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverworldLevel {
    cell_palette: cell::CellPalette<cell::OverworldCellType>,
    name: &'static str,
    state: OverworldLevelState,
    old_position: Point<i32>,
    animation_time: f64,
    levels: [usize; 16],
    to_reveal_next: Vec<Point<i32>>,
}
impl Default for OverworldLevel {
    fn default() -> Self {
        let state = OverworldLevelState::default();
        let old_position = state.get_player_position();

        OverworldLevel {
            cell_palette: cell::CellPalette::new(OverworldCellType::full_palette()),
            name: "",
            state,
            old_position,
            animation_time: 0.0,
            levels: [usize::max_value(); 16],
            to_reveal_next: Vec::new(),
        }
    }
}
impl Level for OverworldLevel {
    fn is_finished_animating(&self) -> bool {
        self.animation_time > CowLevel::ANIMATION_TIME + CowLevel::COOLDOWN_TIME
    }
}
impl component::Component for OverworldLevel {
    type DrawArgs = ();
    fn bounding_rect(&self) -> component::Rect {
        CowLevel::BOUNDING_RECT
    }
    fn click(&mut self, point: Point<i32>) -> bool {
        if !self.in_boundary(point) {
            return false;
        }
        if crate::DEBUG && self.cell_palette.click(point) {
            return true;
        }

        let value: cell::OverworldCell = self.cell_palette.value().into();
        self.state.set_cell_at_cursor(point, value.clone());

        true
    }
    fn draw(&self, context: &Context2D, assets: &Assets, _args: ()) {
        let anim_progress = util::clamp(self.animation_time / CowLevel::ANIMATION_TIME, 0.0, 1.0);
        self.fill_bg(context, super::BG_FILL);

        self.state
            .draw(context, assets, (self.old_position, anim_progress));

        if crate::DEBUG {
            self.cell_palette.fill_bg(context, cell_cursor::BG_COLOUR);
            self.cell_palette.draw(context, assets, ());
        }
    }
    fn step(&mut self, dt: f64, keyboard_state: &KeyboardState) -> NextScene {
        self.animation_time += dt;

        if !self.to_reveal_next.is_empty() {
            return self.reveal();
        }

        if keyboard_state.is_pressed("KeyL") {
            self.log_level()
        }

        if let Some(command) = self.get_keyboard_command(keyboard_state) {
            self.old_position = self.state.get_player_position();
            if command.is_space() {
                match self.current_cell() {
                    OverworldCell::Level(id, _) => {
                        let next_level = self.levels[usize::from(*id)];
                        return NextScene::Call(next_level, Object::Null);
                    },
                    OverworldCell::Finish => {
                        return NextScene::Return(Object::Bool(true));
                    }
                    OverworldCell::Empty 
                    | OverworldCell::Fence(_)
                    | OverworldCell::Wall(_) 
                    | OverworldCell::BlockedPath(_) 
                    | OverworldCell::ClearPath(_) => {}
                }
            }
            self.state.command(command);

            self.animation_time = 0.0;
        };

        NextScene::Continue
    }

    fn returned_into(&mut self, object: Object) {
        assert!(self.to_reveal_next.is_empty());
        if let Object::Bool(true) = object {
            let point = self.state.get_player_position();
            Self::add_adjacents(&mut self.to_reveal_next, point);
        }
    }
    fn called_into(&mut self, _object: Object) {
        self.restore_state();
    }
}
impl OverworldLevel {
    const CELL_REVEAL_TIME: f64 = 300.0;
    fn log_level(&self) {
        crate::console_log!("{}", ron::ser::to_string(&self.state).unwrap());
    }
    pub fn from_data(name: &'static str, string: &str, connections: [usize; 16]) -> Self {
        let state: OverworldLevelState = ron::de::from_str(string).unwrap();
        let position = state.get_player_position();

        OverworldLevel {
            cell_palette: cell::CellPalette::new(OverworldCellType::full_palette()),
            name,
            state,
            old_position: position,
            animation_time: 0.0,
            levels: connections,
            to_reveal_next: Vec::new(),
        }
    }
    fn restore_state(&mut self) {
        assert!(self.to_reveal_next.is_empty());
        let local_storage = util::get_storage();

        match local_storage.get_item(self.name) {
            Err(_) => crate::console_error!("Could not access local storage"),
            Ok(None) => {},
            Ok(Some(string)) => {
                let state: OverworldLevelState = ron::de::from_str(&string).unwrap();
                let position = state.get_player_position();
        
                self.state = state;
                self.old_position = position;
                self.animation_time = 0.0;
            },
        }
    }
    fn save_state(&self) {
        let local_storage = util::get_storage();
        let state_str = ron::ser::to_string(&self.state).unwrap();

        if local_storage.set_item(self.name, &state_str).is_err() {
            crate::console_error!("Could not save to local storage");
        }
    }
    fn current_cell(&self) -> &cell::OverworldCell {
        self.state.get_current_cell()
    }

    fn reveal(&mut self) -> NextScene {
        if self.animation_time < OverworldLevel::CELL_REVEAL_TIME {
            return NextScene::Continue;
        }

        let mut new_reveals = Vec::new();

        self.animation_time = 0.0;
        for point in self.to_reveal_next.iter() {
            let cell = self.state.get_cell(point);
            if cell.can_be_cleared() {
                self.state
                    .set_cell_at_index(*point, OverworldCell::ClearPath(Surroundings::new()));
                Self::add_adjacents(&mut new_reveals, *point);
            }
        }

        if new_reveals.is_empty() {
            self.save_state();
        }

        self.to_reveal_next = new_reveals;
        NextScene::Continue
    }
    fn add_adjacents(vector: &mut Vec<Point<i32>>, point: Point<i32>) {
        Direction::for_every(|direction| {
            let mut adjacent = point;
            adjacent.increment_2d(direction);
            vector.push(adjacent);
        });
    }
}
