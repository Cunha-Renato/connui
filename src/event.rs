use crate::types::{Node, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputEvent {
    Mouse(MouseInputEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseInputEvent {
    Button { button: MouseButton, pressed: bool },
    Move(Point<i16>),
    Scroll(Point<i16>),
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseButton {
    Left = 0b1,
    Right = 0b10,
    Middle = 0b100,
    Forward = 0b1000,
    Backwad = 0b10000,
}
impl MouseButton {
    pub const ALL: [Self; 5] = [
        Self::Left,
        Self::Right,
        Self::Middle,
        Self::Forward,
        Self::Backwad,
    ];
}

#[derive(Default, Debug, Clone)]
pub struct InputState {
    mouse_buttons: u8,
    mouse_position: Point<i16>,
    mouse_scroll: Point<i16>,
}
impl InputState {
    #[inline]
    pub fn mouse_position(&self) -> Point<i16> {
        self.mouse_position
    }

    #[inline]
    pub fn mouse_scroll(&self) -> Point<i16> {
        self.mouse_scroll
    }

    #[inline]
    pub fn mouse_button(&self, button: MouseButton) -> bool {
        self.mouse_buttons & button as u8 != 0
    }

    #[inline]
    pub(crate) fn event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Mouse(mouse_event) => match mouse_event {
                MouseInputEvent::Button { button, pressed } => {
                    if pressed {
                        self.mouse_buttons |= button as u8;
                    } else {
                        self.mouse_buttons &= !(button as u8)
                    }
                }
                MouseInputEvent::Move(point) => self.mouse_position = point,
                MouseInputEvent::Scroll(point) => self.mouse_scroll = point,
            },
        }
    }
}

pub enum Event {
    Mouse {
        event: MouseEvent,
        position: Point<i16>,
    },
}
impl Event {
    pub(crate) fn generate<T>(
        prev_state: &InputState,
        curr_state: &InputState,
        node: &Node<T>,
    ) -> Vec<Self> {
        let mut result = vec![];

        // Cursor is inside the node.
        if node.is_point_inside(curr_state.mouse_position) {
            result.push(Self::Mouse {
                event: MouseEvent::Hover,
                position: curr_state.mouse_position(),
            });

            result.extend(MouseButton::ALL.into_iter().filter_map(|button| {
                let prev_pressed = prev_state.mouse_button(button);
                let curr_pressed = curr_state.mouse_button(button);

                match (prev_pressed, curr_pressed) {
                    (true, true) => Some(MouseEvent::Hold(button)),
                    (true, false) => Some(MouseEvent::Release(button)),
                    (false, true) => Some(MouseEvent::Press(button)),
                    _ => None,
                }
                .map(|e| Self::Mouse {
                    event: e,
                    position: curr_state.mouse_position(),
                })
            }));
        }

        result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseEvent {
    Hover,
    Press(MouseButton),
    Release(MouseButton),
    Hold(MouseButton),
}
