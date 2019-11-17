#[derive(Debug, Clone, Copy)]
enum TimeDirection {
    Forward,
    Backward,
}

/// Invariant: If the time direction is backwards, the state_stack must have at
/// least one element in it.
#[derive(Debug, Clone)]
pub struct StateStack<T> {
    // The stack is one element plus a vector of elements. By construction, This
    // prevents the stack from ever being empty.
    state_stack: Vec<T>,
    stack_top: T,
    time_direction: TimeDirection,
}
impl<T> StateStack<T> {
    pub fn new(state: T) -> Self {
        StateStack {
            state_stack: Vec::new(),
            stack_top: state,
            time_direction: TimeDirection::Forward,
        }
    }
    pub fn push_state(&mut self, state: T) {
        match self.time_direction {
            TimeDirection::Forward => {
                let old_state = std::mem::replace(&mut self.stack_top, state);
                self.state_stack.push(old_state);
            }
            // if the time direction is backward, the current state is actually
            // on the top of the old_states stack, rather than in self.stack
            TimeDirection::Backward => {
                assert!(!self.state_stack.is_empty());
                self.stack_top = state;
                self.time_direction = TimeDirection::Forward;
            }
        }
    }

    pub fn pop_state(&mut self) {
        match self.time_direction {
            // The current state is actually on the top of the old_state stack
            // rather than in self.state when the time direction is backwards.
            // So all that's required to pop the state is to change the time
            // direction
            TimeDirection::Forward => {
                if !self.state_stack.is_empty() {
                    self.time_direction = TimeDirection::Backward;
                }
            }
            TimeDirection::Backward => self.stack_top = self.state_stack.pop().unwrap(),
        }
        if self.state_stack.is_empty() {
            self.time_direction = TimeDirection::Forward;
        }
    }

    pub fn purge_states(&mut self) {
        if let Some(state) = self.state_stack.get_mut(0) {
            std::mem::swap(state, &mut self.stack_top);
        }

        self.state_stack = Vec::new();
        self.time_direction = TimeDirection::Forward;
    }

    /// This returns a reference to the last state, That is, the last current
    /// state before this one.
    ///
    /// If `pop_state` is used, then the last state will actually be on the head
    /// of the stack, and the current state will be second from the top.
    pub fn last_state(&self) -> &T {
        match self.time_direction {
            TimeDirection::Forward => self.state_stack.last().unwrap_or(&self.stack_top),
            TimeDirection::Backward => &self.stack_top,
        }
    }

    /// returns a reference to the current state.
    pub fn current_state(&self) -> &T {
        match self.time_direction {
            TimeDirection::Forward => &self.stack_top,
            TimeDirection::Backward => self.state_stack.last().unwrap(),
        }
    }

    pub fn current_state_mut(&mut self) -> &mut T {
        match self.time_direction {
            TimeDirection::Forward => &mut self.stack_top,
            TimeDirection::Backward => self.state_stack.last_mut().unwrap(),
        }
    }
}
