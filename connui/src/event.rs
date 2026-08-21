use bitflags::bitflags;

use crate::{layout::LayoutElement, renderer::Renderer, types::*, widget::Element};

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

#[derive(Default, Debug, Clone)]
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

#[derive(Clone, Copy)]
pub enum Event {
    Mouse {
        event: MouseEvent,
        position: LPoint<i32>,
    },
}
impl Event {
    pub(crate) fn generate<T, R: Renderer>(
        input_event: InputEvent,

        prev_state: &InputState,
        curr_state: &InputState,

        element: &mut Element<T, R>,
        layout_element: &LayoutElement,

        responses: &mut Vec<T>,
    ) -> bool {
        match input_event {
            InputEvent::Mouse(mouse_input_event) => {
                // Children First.
                let prev_inside = layout_element.clip.is_inside(
                    prev_state.mouse_position.x.as_float(),
                    prev_state.mouse_position.y.as_float(),
                );
                let curr_inside = layout_element.clip.is_inside(
                    curr_state.mouse_position.x.as_float(),
                    curr_state.mouse_position.y.as_float(),
                );

                let response = match mouse_input_event {
                    MouseInputEvent::Button { button, pressed } => {
                        let mouse_event = if pressed {
                            MouseEvent::Press(button)
                        } else {
                            MouseEvent::Release(button)
                        };

                        element.on_event(
                            Self::Mouse {
                                event: mouse_event,
                                position: curr_state.mouse_position,
                            },
                            layout_element,
                        )
                    }
                    MouseInputEvent::Move(point) => {
                        let event = if prev_inside && !curr_inside {
                            MouseEvent::Leave
                        } else if !prev_inside && curr_inside {
                            MouseEvent::Enter
                        } else {
                            MouseEvent::Move
                        };

                        element.on_event(
                            Self::Mouse {
                                event,
                                position: point,
                            },
                            layout_element,
                        )
                    }
                    MouseInputEvent::Scroll(delta) => element.on_event(
                        Self::Mouse {
                            event: MouseEvent::Scroll(delta),
                            position: curr_state.mouse_position,
                        },
                        layout_element,
                    ),
                };

                if let Some(response_result) = response.take() {
                    responses.push(response_result);

                    true
                } else {
                    false
                }
            }
        }
    }

    pub(crate) fn generate_recursive<T, R: Renderer>(
        input_event: InputEvent,

        prev_state: &InputState,
        curr_state: &InputState,

        element: &mut Element<T, R>,
        layout_element: &LayoutElement,

        responses: &mut Vec<T>,
    ) -> bool {
        // Capture
        if let Some((capture_element, capture_layout_element)) =
            element.input_capture(layout_element)
            && matches!(input_event, InputEvent::Mouse(_))
        {
            return Self::generate(
                input_event,
                prev_state,
                curr_state,
                capture_element,
                capture_layout_element,
                responses,
            );
        }

        for (child_element, child_layout_element) in element
            .get_children_mut()
            .iter_mut()
            .zip(&layout_element.children)
        {
            // If true the event was consumed.
            if Self::generate_recursive(
                input_event,
                prev_state,
                curr_state,
                child_element,
                child_layout_element,
                responses,
            ) {
                return true;
            }
        }

        Self::generate(
            input_event,
            prev_state,
            curr_state,
            element,
            layout_element,
            responses,
        )
    }
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
