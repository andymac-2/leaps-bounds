use serde::{Deserialize, Serialize};

use crate::board::Board;
use crate::cell::{Cell, Colour, Direction};
use crate::util::interpolate;
use crate::{Context2D, Point, SpriteSheet};

#[derive(Clone, Debug, Copy, Eq, PartialEq)]
pub enum Command {
    Auto,
    Halt,
    Walk(Direction),
    PlaceBlock(Colour),
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
    pub fn new(player: usize, cow_data: Vec<(Point<i32>, Direction, Vec<usize>)>) -> Self {
        let mut parent_vec = vec![true; cow_data.len()];
        parent_vec[player] = false;

        let cows = cow_data
            .into_iter()
            .map(|(position, direction, children)| {
                children
                    .iter()
                    .for_each(|child_index| parent_vec[*child_index] = false);

                let children_indices = children.into_iter().map(CowIndex).collect();
                Cow::new(position, direction, children_indices)
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

    fn get_cow(&self, cow_index: CowIndex) -> &Cow {
        &self.cows[cow_index.0]
    }

    fn get_cow_mut(&mut self, cow_index: CowIndex) -> &mut Cow {
        &mut self.cows[cow_index.0]
    }

    fn command(&mut self, cow_index: CowIndex, board: &mut Board, command: Command) {
        self.update_children(cow_index, board);
        let cow = self.get_cow_mut(cow_index);

        cow.animation = (cow.animation + 1) % Cow::TOTAL_ANIMATION_FRAMES;
        match command {
            Command::Auto => {
                let cell = cow.get_cell(board);
                match cell {
                    Cell::Empty | Cell::ColouredBlock(_) | Cell::ArrowBlock(_) => {
                        cow.walk_straight(board)
                    }
                    Cell::Arrow(direction) => cow.walk(board, direction),
                    Cell::ColouredArrow(colour, direction) => {
                        self.conditional_walk(cow_index, board, colour, direction)
                    }
                };
            }
            Command::Halt => {}
            Command::Walk(direction) => cow.walk(board, direction),
            Command::PlaceBlock(colour) => {
                board.set_cell(cow.position, Cell::ColouredBlock(colour));
            }
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
            Cell::Empty => Command::Halt,
            Cell::ColouredBlock(colour) => Command::PlaceBlock(colour),
            Cell::Arrow(_) => Command::Halt,
            Cell::ArrowBlock(direction) => Command::Walk(direction),
            Cell::ColouredArrow(_, _) => Command::Halt,
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
            self.get_cow(*child_index).get_cell(board) == Cell::ColouredBlock(colour)
        });

        if is_correct_colour {
            self.get_cow_mut(cow_index).walk(board, direction);
        } else {
            self.get_cow_mut(cow_index).walk_straight(board);
        }
    }

    pub fn draw(
        &self,
        context: &Context2D,
        sprite_sheet: &SpriteSheet,
        old_cows: &Cows,
        anim_progress: f64,
    ) {
        self.cows.iter().enumerate().for_each(|(index, cow)| {
            let old_position = old_cows
                .cows
                .get(index)
                .map_or(cow.position, |old_cow| old_cow.position);
            cow.draw(context, sprite_sheet, old_position, anim_progress);
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Cow {
    position: Point<i32>,
    direction: Direction,
    animation: u32,
    children: Vec<CowIndex>,
}

impl Cow {
    const TOTAL_ANIMATION_FRAMES: u32 = 4;
    const INITIAL_ANIMATION_FRAME: u32 = 0;
    pub fn new(position: Point<i32>, direction: Direction, children: Vec<CowIndex>) -> Self {
        Cow {
            position,
            direction,
            animation: Cow::INITIAL_ANIMATION_FRAME,
            children,
        }
    }

    fn get_cell(&self, board: &Board) -> Cell {
        *board.get_cell(&self.position)
    }

    // Unused argument board: see TODO.
    fn walk(&mut self, board: &Board, direction: Direction) {
        // TODO: check for collisions;
        self.position.increment_2d(direction);
        self.direction = direction;
    }

    fn walk_straight(&mut self, board: &Board) {
        self.walk(board, self.direction);
    }

    pub fn draw(
        &self,
        context: &Context2D,
        sprite_sheet: &SpriteSheet,
        old_position: Point<i32>,
        anim_progress: f64,
    ) {
        let x = interpolate(
            old_position.x().into(),
            self.position.x().into(),
            anim_progress,
        );
        let y = interpolate(
            old_position.y().into(),
            self.position.y().into(),
            anim_progress,
        );

        let sprite_index = Point(self.animation, self.direction.to_index());
        let screen_position = Point(
            x * f64::from(SpriteSheet::STANDARD_WIDTH),
            y * f64::from(SpriteSheet::STANDARD_HEIGHT),
        );

        sprite_sheet.draw(context, sprite_index, screen_position);
    }
}
