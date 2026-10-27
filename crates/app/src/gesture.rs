//! Turns raw press/move/release events into taps and drags.

use macroquad::math::Vec2;

/// Movement in logical pixels before a press becomes a drag.
pub const DRAG_THRESHOLD: f32 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// Reported at the press point, so a slight wobble does not miss the target.
    Tap(Vec2),
    DragStart {
        origin: Vec2,
        at: Vec2,
    },
    DragMove(Vec2),
    Drop(Vec2),
}

#[derive(Debug, Default)]
pub struct Recognizer {
    state: State,
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Idle,
    Pressed(Vec2),
    Dragging,
}

impl Recognizer {
    pub fn press(&mut self, p: Vec2) {
        self.state = State::Pressed(p);
    }

    pub fn moved(&mut self, p: Vec2) -> Option<Gesture> {
        match self.state {
            State::Pressed(origin) if origin.distance(p) > DRAG_THRESHOLD => {
                self.state = State::Dragging;
                Some(Gesture::DragStart { origin, at: p })
            }
            State::Dragging => Some(Gesture::DragMove(p)),
            State::Idle | State::Pressed(_) => None,
        }
    }

    pub fn release(&mut self, p: Vec2) -> Option<Gesture> {
        match std::mem::take(&mut self.state) {
            State::Pressed(origin) => Some(Gesture::Tap(origin)),
            State::Dragging => Some(Gesture::Drop(p)),
            State::Idle => None,
        }
    }

    #[must_use]
    pub fn is_dragging(&self) -> bool {
        matches!(self.state, State::Dragging)
    }
}

#[cfg(test)]
#[path = "tests/gesture.rs"]
mod tests;
