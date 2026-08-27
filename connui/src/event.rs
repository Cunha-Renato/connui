use bitflags::bitflags;

use crate::{
    renderer::Renderer,
    tree::{LayoutElementTree, VisualElement},
    types::*,
};

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
                MouseEvent::Enter | MouseEvent::Left | MouseEvent::Move(_) => EventKind::MOVE,
                MouseEvent::Press(_) | MouseEvent::Release(_) => EventKind::BUTTON,
                MouseEvent::Scroll(_) => EventKind::SCROLL,
                _ => EventKind::empty(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseEvent {
    EnteredWindow,
    LeftWindow,
    Enter,
    Left,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum MousePositionState {
    /// Prev position was outside, current position is inside.
    Enter,
    /// Oposite to [`ENTER`][Self::ENTER].
    Left,
    /// Prev and current position inside.
    Hover,
}
impl MousePositionState {
    fn new(prev: bool, curr: bool) -> Option<Self> {
        let relation = match (prev, curr) {
            (true, true) => Self::Hover,
            (true, false) => Self::Left,
            (false, true) => Self::Enter,
            (false, false) => return None,
        };

        Some(relation)
    }
}

#[derive(Default)]
pub struct InputContext {
    buffer: Vec<Event>,
    prev_mouse_pos: Option<LPoint<i32>>,
    curr_mouse_pos: Option<LPoint<i32>>,
    pub(crate) capture: EventKind,
}
impl InputContext {
    pub(crate) fn next_frame(&mut self) {
        self.prev_mouse_pos = self.curr_mouse_pos;
    }

    #[inline]
    pub(crate) fn process_incoming(&mut self, event: InputEvent) {
        let mouse_event = match event {
            InputEvent::Mouse(mouse_input_event) => match mouse_input_event {
                MouseInputEvent::Button { button, pressed } => {
                    if pressed {
                        MouseEvent::Press(button)
                    } else {
                        MouseEvent::Release(button)
                    }
                }
                MouseInputEvent::Move(point) => {
                    self.curr_mouse_pos = Some(point);

                    MouseEvent::Move(point)
                }
                MouseInputEvent::Scroll(delta) => MouseEvent::Scroll(delta),
                MouseInputEvent::EnteredWindow => MouseEvent::EnteredWindow,
                MouseInputEvent::LeftWindow => {
                    self.prev_mouse_pos = None;
                    self.curr_mouse_pos = None;

                    MouseEvent::LeftWindow
                }
            },
        };

        self.buffer.push(Event::Mouse(mouse_event));
    }

    fn mouse_pos_state(&self, rect: &LRect<f32, f32>) -> Option<MousePositionState> {
        let prev_inside = self
            .prev_mouse_pos
            .map(|pp| rect.is_inside(pp.x.as_float(), pp.y.as_float()))
            .unwrap_or_default();

        let curr_inside = self
            .curr_mouse_pos
            .map(|pp| rect.is_inside(pp.x.as_float(), pp.y.as_float()))
            .unwrap_or_default();

        MousePositionState::new(prev_inside, curr_inside)
    }
}
impl InputContext {
    pub(crate) fn propagate_input<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
    ) -> Vec<T> {
        if self.buffer.is_empty() {
            return vec![];
        }

        let mut responses = vec![];

        for event in std::mem::take(&mut self.buffer) {
            println!("{event:#?}");

            // Capture first
            if self.capture.intersects(event.kind())
                && self.handle_capture(event, root, layout_tree, &mut responses)
            {
                continue;
            }

            match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    MouseEvent::Move(_) => {
                        let mut enter = false;
                        let mut left = false;
                        let mut finish = false;

                        // Maybe use move position?
                        self.handle_mouse_move(
                            root,
                            layout_tree,
                            &mut responses,
                            &mut enter,
                            &mut left,
                            &mut finish,
                        )
                    }
                    _ => {
                        self.handle_all(event, root, layout_tree, &mut responses);
                    }
                },
            }
        }

        self.next_frame();
        responses
    }

    fn handle_capture<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        root.find_element(&mut |el| self.capture.intersects(el.element.input_capture()))
            .is_some_and(|element| {
                let consumed = self.handle_individual(event, element, layout_tree, responses);

                if consumed {
                    self.capture = self.capture.intersection(element.element.input_capture());
                }

                consumed
            })
    }

    /// Returns **true** if the event was consumed.
    fn handle_individual<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        element: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        let layout_element = &layout_tree[element.layout_key];

        let response = element.element.input_event(event, layout_element);
        let consumed = response.consumed();

        if let Some(msg) = response.take() {
            responses.push(msg);
        }

        // Only (re)register capture if this element actually did something with
        // the event, so capture state doesn't get silently re-armed every frame
        // regardless of whether anything happened.
        if consumed {
            self.capture.insert(element.element.input_capture());
        }

        consumed
    }

    fn handle_all<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        // Children first.
        for child in root.children.iter_mut().rev() {
            if self.handle_all(event, child, layout_tree, responses) {
                return true;
            }
        }

        self.handle_individual(event, root, layout_tree, responses)
    }

    fn handle_mouse_move<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
        enter: &mut bool,
        left: &mut bool,
        finish: &mut bool,
    ) {
        for child in root.children.iter_mut().rev() {
            self.handle_mouse_move(child, layout_tree, responses, enter, left, finish);
        }

        if *finish {
            return;
        }

        let layout_element = &layout_tree[root.layout_key];
        let event = if let Some(mpos_state) = self.mouse_pos_state(&layout_element.clip) {
            match mpos_state {
                MousePositionState::Enter => {
                    *enter = true;
                    MouseEvent::Enter
                }
                MousePositionState::Left => {
                    *left = true;
                    MouseEvent::Left
                }
                MousePositionState::Hover => {
                    *finish = true;

                    // Signal that the cursor left from this widget.
                    if *enter {
                        MouseEvent::Left
                    } else if *left {
                        MouseEvent::Enter
                    } else {
                        MouseEvent::Move(self.curr_mouse_pos.unwrap())
                    }
                }
            }
        } else {
            return;
        };

        if let Some(response) = root
            .element
            .input_event(Event::Mouse(event), layout_element)
            .take()
        {
            responses.push(response);
        }
    }
}
