use bitflags::bitflags;

use crate::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputEvent {
    Mouse(MouseInputEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseInputEvent {
    EnteredWindow,
    LeftWindow,
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

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Mouse(MouseEvent),
}
impl Event {
    pub const fn kind(&self) -> EventKind {
        match self {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::EnteredWindow | MouseEvent::LeftWindow | MouseEvent::Move(_) => {
                    EventKind::MOVE
                }
                MouseEvent::Press(_) | MouseEvent::Release(_) => EventKind::BUTTON,
                MouseEvent::Scroll(_) => EventKind::SCROLL,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseEvent {
    EnteredWindow,
    LeftWindow,
    Move(LPoint<i32>),
    Scroll(LPoint<i32>),
    Press(MouseButton),
    Release(MouseButton),
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct EventKind: u8 {
        const MOVE = 0b1;
        const BUTTON = 0b10;
        const SCROLL = 0b100;
    }
}

#[derive(Default)]
pub struct InputContext {
    buffer: Vec<Event>,
    mouse_pos: LPoint<i32>,
    pub(crate) capture: EventKind,
}
impl InputContext {
    #[inline]
    pub(crate) fn events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.buffer)
    }

    #[inline]
    pub(crate) fn process_incoming(&mut self, event: InputEvent) {
        match event {
            InputEvent::Mouse(mouse_input_event) => match mouse_input_event {
                MouseInputEvent::Button { button, pressed } => {
                    let mouse_event = if pressed {
                        MouseEvent::Press(button)
                    } else {
                        MouseEvent::Release(button)
                    };

                    self.buffer.extend([
                        Event::Mouse(mouse_event),
                        Event::Mouse(MouseEvent::Move(self.mouse_pos)),
                    ]);
                }
                MouseInputEvent::Move(point) => {
                    self.mouse_pos = point;

                    self.buffer.push(Event::Mouse(MouseEvent::Move(point)));
                }
                MouseInputEvent::Scroll(delta) => {
                    self.buffer.push(Event::Mouse(MouseEvent::Scroll(delta)))
                }
                MouseInputEvent::EnteredWindow => {
                    self.buffer.push(Event::Mouse(MouseEvent::EnteredWindow))
                }
                MouseInputEvent::LeftWindow => {
                    self.buffer.push(Event::Mouse(MouseEvent::LeftWindow))
                }
            },
        };
    }
}
