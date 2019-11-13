use crate::{Context2D, Assets};
use crate::point::Point;
use crate::util::with_saved_context;

// invariant: dimensions are positive
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub top_left: Point<i32>,
    pub dimensions: Point<i32>,
}
impl Rect {
    pub fn expand(&self, increase: Point<i32>) -> Rect {
        Rect {
            top_left: self.top_left - increase,
            dimensions: self.dimensions + increase + increase
        }
    }
}

pub trait Component {
    type Args;
    // performs a click event on a given component. returns true if the event
    // was handled.
    fn click (&mut self, point: Point<i32>) -> bool {
        false
    }
    /// Default behaviour assumes an AABB
    fn in_boundary(&self, point: Point<i32>) -> bool {
        let Rect { top_left, dimensions } = self.bounding_rect();
        let local_point = point - top_left;

        local_point.x() >= 0 
            && local_point.x() < dimensions.x() 
            && local_point.y() >= 0
            && local_point.y() < dimensions.y()
    }
    fn top_left(&self) -> Point<i32> {
        self.bounding_rect().top_left
    }
    fn dimensions (&self) -> Point<i32> {
        self.bounding_rect().dimensions
    }
    fn bounding_rect (&self) -> Rect;
    fn draw (&self, context: &Context2D, assets: &Assets, args: Self::Args);
    fn draw_bbox(&self, context: &Context2D, assets: &Assets, args: Self::Args) {
        self.draw(context, assets, args);
        let dimensions = self.dimensions();
        let top_left = self.top_left();
        context.rect(top_left.x().into(), top_left.y().into(), dimensions.x().into(), dimensions.y().into());
        context.stroke();
    }
    fn combine_dimensions<C: Component>(&self, other: &C) -> Rect {
        let self_tl = self.top_left();
        let other_tl = other.top_left();

        let self_br = self_tl + self.dimensions();
        let other_br = other_tl + other.dimensions();

        let tl_x = self_tl.x().min(other_tl.x());
        let tl_y = self_tl.y().min(other_tl.y());

        let br_x = self_br.x().max(other_br.x());
        let br_y = self_br.y().max(other_br.y());

        Rect {
            top_left: Point(tl_x, tl_y), 
            dimensions: Point(br_x - tl_x, br_y - tl_y),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Translation<T> {
    pub translation: Point<i32>,
    pub component: T,
}
impl<T> Translation<T> {
    pub fn new(translation: Point<i32>, component: T) -> Self {
        Translation {
            translation,
            component,
        }
    }
    fn get_local_point(&self, point: Point<i32>) -> Point<i32> {
        point - self.translation
    }
}
impl <T> std::ops::Deref for Translation<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.component
    }
}
impl <T> std::ops::DerefMut for Translation<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
impl<T: Component> Component for Translation<T> {
    type Args = T::Args;
    fn click(&mut self, point: Point<i32>) -> bool {
        if !Component::in_boundary(self, point) {
            return false;
        }
        let local_point = self.get_local_point(point);

        self.component.click(local_point)
    }
    fn bounding_rect(&self) -> Rect {
        Rect {
            top_left: self.translation + self.component.top_left(),
            dimensions: self.component.dimensions(),
        }
    }
    fn draw(&self, context: &Context2D, assets: &Assets, args: Self::Args) {
        with_saved_context(context, || {
            context.translate(self.translation.x().into(), self.translation.y().into());
            self.component.draw(context, assets, args);
        });
    }
}