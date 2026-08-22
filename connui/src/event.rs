use bitflags::bitflags;

use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputEvent {
    Mouse(MouseInputEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseInputEvent {
    Button { button: MouseButton, pressed: bool },
    Move(LPoint<i32>),
    Scroll(LPoint<i32>),
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MouseButton: u8 {
        const LEFT = 0b1;
        const RIGHT = 0b10;
        const MIDDLE = 0b100;
        const FORWARD = 0b1000;
        const BACKWARD = 0b10000;
    }
}
impl MouseButton {
    pub const ALL: [Self; 5] = [
        Self::LEFT,
        Self::RIGHT,
        Self::MIDDLE,
        Self::FORWARD,
        Self::BACKWARD,
    ];
}

#[derive(Clone, Copy)]
pub enum Event {
    Mouse {
        event: MouseEvent,
        position: LPoint<i32>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseEvent {
    Enter,
    Leave,
    Move,
    Scroll(LPoint<i32>),
    Press(MouseButton),
    Release(MouseButton),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct InputState {
    mouse_position: LPoint<i32>,
    mouse_buttons: MouseButton,
}
impl InputState {
    #[inline]
    pub const fn mouse_position(&self) -> LPoint<i32> {
        self.mouse_position
    }

    #[inline]
    pub const fn mouse_button(&self, button: MouseButton) -> bool {
        self.mouse_buttons.contains(button)
    }

    pub(crate) fn event(&mut self, event: InputEvent) {
        match event {
            InputEvent::Mouse(mouse_event) => match mouse_event {
                MouseInputEvent::Button { button, pressed } => {
                    if pressed {
                        self.mouse_buttons.insert(button);
                    } else {
                        self.mouse_buttons.remove(button);
                    }
                }
                MouseInputEvent::Move(point) => self.mouse_position = point,
                _ => {}
            },
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InputCapture: u8 {
        const MOVE = 0b1;
        const BUTTON = 0b10;
        const SCROLL = 0b100;
    }
}

pub struct InputContext {
    prev_state: InputState,
    curr_state: InputState,
    capture: InputCapture,
    focus: bool,
}
impl InputContext {
    pub(crate) fn new_frame(&mut self) {
        self.prev_state = self.curr_state;
    }
}
