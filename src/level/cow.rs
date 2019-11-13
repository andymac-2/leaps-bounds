use serde::{Deserialize, Serialize};

use crate::direction::Direction;
use crate::point::interpolate_2d;
use crate::{console_log, Context2D, Point, SpriteSheet};

use super::board::Board;
use super::cell::{Colour, GroundCell};
use super::{LevelState, SuccessState};

#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum CowSprite {
    White = 0,
    Grey = 1,
}

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Command {
    Auto,
    Halt,
    Walk(Direction),
    PlaceBlock(Colour),
    RotateRight,
    RotateLeft,
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct CowIndex(usize);
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cows {
    player: CowIndex,
    parents: Vec<CowIndex>,
    cows: Vec<Cow>,
}
impl Cows {
    pub fn new(player: usize, cow_data: Vec<(Point<i32>, Direction, CowSprite, Vec<usize>)>) -> Self {
        let mut parent_vec = vec![true; cow_data.len()];
        parent_vec[player] = false;

        let cows = cow_data
            .into_iter()
            .map(|(position, direction, sprite, children)| {
                children
                    .iter()
                    .for_each(|child_index| parent_vec[*child_index] = false);

                let children_indices = children.into_iter().map(CowIndex).collect();
                Cow::new(position, direction, children_indices, sprite)
            })
            .collect();

        let parents = parent_vec
            .into_iter()
            .enumerate()
            .filter(|(_, is_parent)| *is_parent)
            .map(|(index, _)| CowIndex(index))
            .collect();

        Cows {
            player: CowIndex(player),
            parents,
            cows,
        }
    }

    pub fn command_player(&mut self, board: &mut Board, command: Command) {
        self.command(self.player, board, command);

        // CIRCULAR REFERENCE WARNING !!!! The parents vector is cached here.
        // It is assumed that the parents are unmodified through the process of
        // updating the children. Breaking this assumption may lead to bugs.
        let parents = self.parents.clone();
        for cow_index in parents {
            self.command(cow_index, board, Command::Auto);
        }
    }

    pub fn success_state(&self, board: &Board) -> SuccessState {
        let mut acc = SuccessState::Succeeded;
        for cow in self.cows.iter() {
            acc.combine(board.get_overlay_cell(&cow.position).success_state())
        };
        acc
    }

    fn get_cow(&self, cow_index: CowIndex) -> &Cow {
        &self.cows[cow_index.0]
    }

    fn get_cow_mut(&mut self, cow_index: CowIndex) -> &mut Cow {
        &mut self.cows[cow_index.0]
    }

    fn command(&mut self, cow_index: CowIndex, board: &mut Board, command: Command) {
        self.update_children(cow_index, board);
        let cow = self.get_cow_mut(cow_index);

        match command {
            Command::Auto => {
                let cell = cow.get_cell(board);
                match cell {
                    GroundCell::Empty
                    | GroundCell::ColouredBlock(_)
                    | GroundCell::ArrowBlock(_)
                    | GroundCell::RotateLeft
                    | GroundCell::RotateRight
                    | GroundCell::Fence(_)
                    | GroundCell::Wall(_) => cow.walk_bounce(board),
                    GroundCell::Arrow(direction) => cow.walk_stop(board, direction),
                    GroundCell::ColouredArrow(colour, direction) => {
                        self.conditional_walk(cow_index, board, colour, direction)
                    }
                };
            }
            Command::Halt => {}
            Command::Walk(direction) => cow.walk_stop(board, direction),
            Command::PlaceBlock(colour) => cow.place_block(board, colour),
            Command::RotateLeft => cow.rotate_block_left(board),
            Command::RotateRight => cow.rotate_block_right(board),
        }
    }

    fn update_children(&mut self, cow_index: CowIndex, board: &mut Board) {
        let cow = self.get_cow(cow_index);
        let cell = cow.get_cell(board);

        // CIRCULAR REFERENCE WARNING !!!! The children vector is cached here.
        // It is assumed that the parent is unmodified through the process of
        // updating the children. Breaking this assumption may lead to bugs.
        let children = cow.children.clone();

        let command = match cell {
            GroundCell::Empty => Command::Halt,
            GroundCell::ColouredBlock(colour) => Command::PlaceBlock(colour),
            GroundCell::Arrow(_) => Command::Halt,
            GroundCell::ArrowBlock(direction) => Command::Walk(direction),
            GroundCell::ColouredArrow(_, _) => Command::Halt,
            GroundCell::RotateRight => Command::RotateRight,
            GroundCell::RotateLeft => Command::RotateLeft,
            GroundCell::Fence(_) => {
                console_log!("WARNING: Cow registered inside Fence");
                Command::Halt
            }
            GroundCell::Wall(_) => {
                console_log!("WARNING: Cow registered inside wall");
                Command::Halt
            }
        };

        children.into_iter().for_each(|child_index| {
            self.command(child_index, board, command);
        });
    }

    fn conditional_walk(
        &mut self,
        cow_index: CowIndex,
        board: &Board,
        colour: Colour,
        direction: Direction,
    ) {
        let is_correct_colour = self.get_cow(cow_index).children.iter().any(|child_index| {
            self.get_cow(*child_index).get_cell(board) == GroundCell::ColouredBlock(colour)
        });

        if is_correct_colour {
            self.get_cow_mut(cow_index).walk_stop(board, direction);
        } else {
            self.get_cow_mut(cow_index).walk_bounce(board);
        }
    }

    fn get_screen_position(
        &self,
        old_cows: &Cows,
        index: CowIndex,
        anim_progress: f64,
    ) -> Point<f64> {
        let new_position = self.get_cow(index).position;
        let old_position = old_cows
            .cows
            .get(index.0)
            .map_or(new_position, |old_cow| old_cow.position);
        let grid_position = interpolate_2d(old_position, new_position, anim_progress);

        grid_position
            * Point(
                f64::from(SpriteSheet::STANDARD_WIDTH),
                f64::from(SpriteSheet::STANDARD_HEIGHT),
            )
    }

    pub fn draw(
        &self,
        context: &Context2D,
        sprite_sheet: &SpriteSheet,
        old_cows: &Cows,
        anim_progress: f64,
        anim_frame: u8,
    ) {
        context.save();

        context
            .translate(
                f64::from(SpriteSheet::STANDARD_WIDTH / 2),
                f64::from(SpriteSheet::STANDARD_HEIGHT / 4),
            )
            .unwrap();

        self.cows.iter().enumerate().for_each(|(index, cow)| {
            let this_position = self.get_screen_position(old_cows, CowIndex(index), anim_progress);
            for index in &cow.children {
                let other_position = self.get_screen_position(old_cows, *index, anim_progress);
                crate::js_ffi::draw_rope(
                    context,
                    this_position.x(),
                    this_position.y(),
                    other_position.x(),
                    other_position.y(),
                );
            }
        });

        context.restore();

        // drow cows
        self.cows.iter().enumerate().for_each(|(index, cow)| {
            let old_position = old_cows
                .cows
                .get(index)
                .map_or(cow.position, |old_cow| old_cow.position);
            cow.draw(
                context,
                sprite_sheet,
                old_position,
                anim_progress,
                anim_frame,
            );
        });
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cow {
    position: Point<i32>,
    direction: Direction,
    children: Vec<CowIndex>,
    sprite: CowSprite,
}

impl Cow {
    pub fn new(position: Point<i32>, direction: Direction, children: Vec<CowIndex>, sprite: CowSprite) -> Self {
        Cow {
            position,
            direction,
            children,
            sprite,
        }
    }

    fn get_cell(&self, board: &Board) -> GroundCell {
        *board.get_ground_cell(&self.position)
    }

    // walk until you hit a wall.
    fn walk_stop(&mut self, board: &Board, direction: Direction) {
        self.direction = direction;

        let mut forwards = self.position;
        forwards.increment_2d(direction);

        if !board.get_ground_cell(&forwards).is_solid_to_cows() {
            self.position.increment_2d(direction);
            return;
        }
    }

    // when you hit a wall, turn around and bounce the other way.
    fn walk_bounce(&mut self, board: &Board) {
        let mut forwards = self.position;
        forwards.increment_2d(self.direction);

        if !board.get_ground_cell(&forwards).is_solid_to_cows() {
            self.position.increment_2d(self.direction);
            return;
        }

        let opposite_dir = self.direction.opposite();
        self.direction = opposite_dir;

        let mut backwards = self.position;
        backwards.increment_2d(opposite_dir);

        if !board.get_ground_cell(&backwards).is_solid_to_cows() {
            self.position.increment_2d(opposite_dir);
        }
    }

    fn place_block(&mut self, board: &mut Board, colour: Colour) {
        board.set_ground_cell(self.position, GroundCell::ColouredBlock(colour));
    }

    fn rotate_block_right(&mut self, board: &mut Board) {
        board.map_ground_cell(self.position, GroundCell::rotate_right);
    }

    fn rotate_block_left(&mut self, board: &mut Board) {
        board.map_ground_cell(self.position, GroundCell::rotate_left);
    }

    fn get_screen_position(&self, other: Point<i32>, anim_progress: f64) -> Point<f64> {
        let grid_position = interpolate_2d(other, self.position, anim_progress);

        grid_position
            * Point(
                f64::from(SpriteSheet::STANDARD_WIDTH),
                f64::from(SpriteSheet::STANDARD_HEIGHT),
            )
    }

    pub fn draw(
        &self,
        context: &Context2D,
        sprite_sheet: &SpriteSheet,
        old_position: Point<i32>,
        anim_progress: f64,
        animation_frame: u8,
    ) {
        let position = self.get_screen_position(old_position, anim_progress);
        let sprite_index = Point(self.direction as u8 * LevelState::TOTAL_ANIMATION_FRAMES + animation_frame, self.sprite as u8);

        sprite_sheet.draw(context, sprite_index, position);
    }
}
