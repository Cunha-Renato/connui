use crate::types::Point;

#[derive(Clone, Copy)]
pub enum InputEvent {
    Mouse(MouseEvent),
}

#[derive(Clone, Copy)]
pub enum MouseEvent {
    Button { button: MouseButton, pressed: bool },
    Move(Point<u16>),
    Scroll(Point<i16>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Front,
    Back,
    Other(u16),
}

#[derive(Default, Debug)]
pub struct InputState {
    mouse_position: Point<u16>,
}
impl InputState {
    #[inline]
    pub fn mouse_position(&self) -> Point<u16> {
        self.mouse_position
    }

    #[inline]
    pub(crate) fn event(&mut self, event: InputEvent) {
        if let InputEvent::Mouse(MouseEvent::Move(point)) = event {
            self.mouse_position = point;
        }
    }
}
