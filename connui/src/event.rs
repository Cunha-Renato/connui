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
impl InputCapture {
    #[inline]
    pub const fn should_capture(&self, mouse_event: MouseEvent) -> bool {
        match mouse_event {
            MouseEvent::Move => self.contains(Self::MOVE),
            MouseEvent::Scroll(_) => self.contains(Self::SCROLL),
            MouseEvent::Press(_) | MouseEvent::Release(_) => self.contains(Self::BUTTON),
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct InputContext {
    buffer: Vec<Event>,
    prev_state: InputState,
    curr_state: InputState,
    pub(crate) capture: Option<InputCapture>,
    pub(crate) focus: bool,
}
impl InputContext {
    #[inline]
    pub(crate) fn new_frame(&mut self) {
        self.buffer.clear();
        self.prev_state = self.curr_state;
    }

    #[inline]
    pub(crate) fn events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.buffer)
    }

    #[inline]
    pub(crate) fn set_capture(&mut self, input_capture: InputCapture) {
        if let Some(capture) = &mut self.capture {
            capture.insert(input_capture);
        }
    }

    #[inline]
    pub(crate) fn set_focus(&mut self) {
        self.focus = true;
    }

    pub(crate) fn process_incoming(&mut self, event: InputEvent) {
        self.curr_state.event(event);

        let mouse_event = match event {
            InputEvent::Mouse(mouse_input_event) => match mouse_input_event {
                MouseInputEvent::Button { button, pressed } => {
                    if pressed {
                        MouseEvent::Press(button)
                    } else {
                        MouseEvent::Release(button)
                    }
                }
                MouseInputEvent::Move(_) => MouseEvent::Move,
                MouseInputEvent::Scroll(delta) => MouseEvent::Scroll(delta),
            },
        };

        self.buffer.push(Event::Mouse {
            event: mouse_event,
            position: self.curr_state.mouse_position(),
        });
    }

    pub(crate) fn gen_synthetic(&self, event: Event, rect: &LRect<f32, f32>) -> Vec<Event> {
        let mut events = vec![event];

        match event {
            Event::Mouse {
                event: MouseEvent::Move,
                position,
            } => {
                let prev_inside = rect.is_inside(
                    self.prev_state.mouse_position().x.as_float(),
                    self.prev_state.mouse_position().y.as_float(),
                );
                let curr_inside = rect.is_inside(
                    self.curr_state.mouse_position().x.as_float(),
                    self.curr_state.mouse_position().y.as_float(),
                );

                if !prev_inside && curr_inside {
                    events.push(Event::Mouse {
                        event: MouseEvent::Enter,
                        position,
                    })
                } else if prev_inside && !curr_inside {
                    events.push(Event::Mouse {
                        event: MouseEvent::Leave,
                        position,
                    })
                }
            }

            _ => {}
        }

        events
    }
}
