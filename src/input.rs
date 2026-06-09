use crate::types::Point;

#[derive(Clone, Copy)]
pub enum InputEvent {
    Mouse(MouseEvent),
}

#[derive(Clone, Copy)]
pub enum MouseEvent {
    Button { button: MouseButton, pressed: bool },
    Move(Point<u16>),
    Scroll(Point<u16>),
}

#[derive(Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Front,
    Back,
    Other(u16),
}
