use bitflags::bitflags;

use crate::{
    renderer::Renderer,
    tree::{LayoutElementTree, VisualElement},
    types::*,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InputEvent {
    Window(WindowEvent),
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

#[derive(Debug, Clone, Copy)]
pub enum Event {
    Window(WindowEvent),
    Mouse(MouseEvent),
}
impl Event {
    pub const fn kind(&self) -> EventKind {
        match self {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::Enter | MouseEvent::Left | MouseEvent::Move(_) => EventKind::MOVE,
                MouseEvent::Press(_) | MouseEvent::Release(_) => EventKind::BUTTON,
                MouseEvent::Scroll(_) => EventKind::SCROLL,
            },
            _ => EventKind::empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowEvent {
    CursorEnter,
    CurserLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MouseEvent {
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
        match event {
            InputEvent::Mouse(mouse_input_event) => {
                let event = match mouse_input_event {
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
                };

                self.buffer.push(Event::Mouse(event));
            }
            InputEvent::Window(window_event) => {
                let event = match window_event {
                    WindowEvent::CursorEnter => WindowEvent::CursorEnter,
                    WindowEvent::CurserLeft => {
                        self.prev_mouse_pos = None;
                        self.curr_mouse_pos = None;

                        WindowEvent::CurserLeft
                    }
                };

                self.buffer.push(Event::Window(event));
            }
        }
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
    pub(crate) fn send_input<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
    ) -> Vec<T> {
        if self.buffer.is_empty() {
            return vec![];
        }

        let mut responses = vec![];

        for event in std::mem::take(&mut self.buffer) {
            let captured = self.capture.intersects(event.kind())
                && self.send_capture(event, root, layout_tree, &mut responses);

            if !captured {
                self.send_all(event, root, layout_tree, &mut responses);
            }
        }

        self.next_frame();
        responses
    }

    fn send_capture<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        let Some(element) =
            root.find_element(&mut |el| self.capture.intersects(el.element.input_capture()))
        else {
            return false;
        };

        let (response, consumed) = match event {
            Event::Window(window_event) => (element.element.window_event(window_event), false),
            Event::Mouse(mouse_event) => {
                let response =
                    self.send_mouse_individual_capture(mouse_event, element, layout_tree);
                let consumed = response.consumed();
                (response.take(), consumed)
            }
        };

        self.capture &= element.element.input_capture();
        responses.extend(response);
        consumed
    }

    fn send_all<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        match event {
            Event::Window(window_event) => {
                self.send_window(window_event, root, responses);
                false
            }
            Event::Mouse(mouse_event) => {
                self.send_mouse(mouse_event, root, layout_tree, responses);
                true
            }
        }
    }

    fn send_window<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: WindowEvent,
        root: &mut VisualElement<T, R>,
        responses: &mut Vec<T>,
    ) {
        for child in root.children.iter_mut() {
            self.send_window(event, child, responses);
        }
        responses.extend(root.element.window_event(event));
    }

    fn send_mouse<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: MouseEvent,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        match event {
            MouseEvent::Move(_) => self.send_mouse_move(root, layout_tree, responses),
            _ => {
                self.send_mouse_inner(event, root, layout_tree, responses);
            }
        }
    }

    fn send_mouse_inner<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: MouseEvent,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        let Some(pos) = self.curr_mouse_pos else {
            return false;
        };
        if !layout_tree[root.layout_key]
            .clip
            .is_inside(pos.x.as_float(), pos.y.as_float())
        {
            return false;
        }

        let hit_child = root
            .children
            .iter_mut()
            .rev()
            .any(|child| self.send_mouse_inner(event, child, layout_tree, responses));

        if hit_child {
            return true;
        }

        responses.extend(root.element.mouse_event(event).take());
        self.capture |= root.element.input_capture();
        true
    }

    fn send_mouse_move<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        let mut state = MoveState::Searching;
        self.send_mouse_move_inner(root, layout_tree, responses, &mut state);
    }

    fn send_mouse_move_inner<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
        responses: &mut Vec<T>,
        state: &mut MoveState,
    ) {
        let Some(pos_state) = self.mouse_pos_state(&layout_tree[root.layout_key].clip) else {
            return;
        };

        for child in root.children.iter_mut().rev() {
            self.send_mouse_move_inner(child, layout_tree, responses, state);
        }

        if matches!(state, MoveState::Done) {
            return;
        }

        let event = match (pos_state, &state) {
            (MousePositionState::Enter, MoveState::Searching) => {
                *state = MoveState::Entered;
                MouseEvent::Enter
            }
            (MousePositionState::Left, MoveState::Searching) => {
                *state = MoveState::Left;
                MouseEvent::Left
            }
            (MousePositionState::Hover, _) => {
                let event = match state {
                    MoveState::Entered => MouseEvent::Left,
                    MoveState::Left => MouseEvent::Enter,
                    _ => MouseEvent::Move(self.curr_mouse_pos.unwrap()),
                };
                *state = MoveState::Done;
                event
            }
            _ => return,
        };

        responses.extend(root.element.mouse_event(event).take());
    }

    fn send_mouse_individual_capture<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: MouseEvent,
        element: &mut VisualElement<T, R>,
        layout_tree: &LayoutElementTree,
    ) -> Response<T> {
        let MouseEvent::Move(_) = event else {
            return element.element.mouse_event(event);
        };

        match self.mouse_pos_state(&layout_tree[element.layout_key].clip) {
            Some(MousePositionState::Enter) => element.element.mouse_event(MouseEvent::Enter),
            Some(MousePositionState::Left) => element.element.mouse_event(MouseEvent::Left),
            _ => element.element.mouse_event(event),
        }
    }
}

/// For mouse_move input.
enum MoveState {
    Searching,
    Entered,
    Left,
    Done,
}
