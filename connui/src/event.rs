use crate::{
    renderer::Renderer,
    types::{LogicalPixel, Node, Point},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputEvent {
    Mouse(MouseInputEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseInputEvent {
    Button { button: MouseButton, pressed: bool },
    Move(Point<LogicalPixel<i16>>),
    Scroll(Point<LogicalPixel<i16>>),
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
    mouse_position: Point<LogicalPixel<i16>>,
    mouse_scroll: Point<LogicalPixel<i16>>,
}
impl InputState {
    #[inline]
    pub fn mouse_position(&self) -> Point<LogicalPixel<i16>> {
        self.mouse_position
    }

    #[inline]
    pub fn mouse_scroll(&self) -> Point<LogicalPixel<i16>> {
        self.mouse_scroll
    }

    #[inline]
    pub fn mouse_button(&self, button: MouseButton) -> bool {
        self.mouse_buttons & button as u8 != 0
    }

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

    #[inline]
    pub(crate) fn next_frame(&mut self) {
        self.mouse_scroll = Point::default();
    }
}

pub enum Event {
    Mouse {
        event: MouseEvent,
        position: Point<LogicalPixel<i16>>,
    },
}
impl Event {
    pub(crate) fn generate<T, R: Renderer>(
        prev_state: &InputState,
        curr_state: &InputState,
        node: &Node<T, R>,
    ) -> Vec<Self> {
        let mut result = vec![];

        // Cursor is inside the node.
        if node.is_point_inside(curr_state.mouse_position) {
            // Hover.
            result.push(Self::Mouse {
                event: MouseEvent::Hover,
                position: curr_state.mouse_position,
            });

            // Scroll.
            if curr_state.mouse_scroll.x != LogicalPixel::new(0)
                || curr_state.mouse_scroll.y != LogicalPixel::new(0)
            {
                result.push(Self::Mouse {
                    event: MouseEvent::Scroll(curr_state.mouse_scroll),
                    position: curr_state.mouse_position,
                });
            }

            // Buttons.
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
    Scroll(Point<LogicalPixel<i16>>),
    Press(MouseButton),
    Release(MouseButton),
    Hold(MouseButton),
}
