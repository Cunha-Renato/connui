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

#[derive(Default)]
pub struct InputContext {
    buffer: Vec<Event>,
    prev_mouse_pos: Option<LPoint<i32>>,
    curr_mouse_pos: Option<LPoint<i32>>,
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
}
impl InputContext {
    pub(crate) fn send_input<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
    ) -> Vec<T> {
        if self.buffer.is_empty() {
            return vec![];
        }

        let mut responses = vec![];

        for event in std::mem::take(&mut self.buffer) {
            let captured = self.send_capture(event, root, layout_tree, &mut responses);

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
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) -> bool {
        let (Some(element), Event::Mouse(mouse_event)) = (
            root.find_element(&mut |el| event.kind().intersects(el.element.input_capture())),
            event,
        ) else {
            return false;
        };

        let element_ptr = element as *mut VisualElement<T, R>;
        let prev_capture = element.element.input_capture();

        let response = element.mouse_event(mouse_event, layout_tree);
        let curr_capture = element.element.input_capture();
        let consumed = response.consumed();
        responses.extend(response.take());

        let lost_move_capture = consumed
            && prev_capture.contains(EventKind::MOVE)
            && !curr_capture.contains(EventKind::MOVE);

        if lost_move_capture {
            self.reconcile_hover_after_capture_loss(root, element_ptr, layout_tree, responses);
        }

        consumed
    }

    fn reconcile_hover_after_capture_loss<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        prev_hover: *mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        let curr_hover = root
            .find_element(&mut |el| {
                let clip = &layout_tree[el.layout_key].clip;

                self.curr_mouse_pos.is_some_and(|position| {
                    clip.is_inside(position.x.as_float(), position.y.as_float())
                })
            })
            .map(|el| el as *mut VisualElement<T, R>);

        if curr_hover.is_some_and(|c| std::ptr::eq(c, prev_hover)) {
            return; // still the same element — nothing changed
        }

        unsafe {
            responses.extend(
                (*prev_hover)
                    .mouse_event(MouseEvent::Left, layout_tree)
                    .take(),
            );
            responses.extend(
                curr_hover
                    .and_then(|curr| (*curr).mouse_event(MouseEvent::Enter, layout_tree).take()),
            );
        }
    }

    fn send_all<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: Event,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        match event {
            Event::Window(window_event) => {
                self.send_window(window_event, root, layout_tree, responses);
            }
            Event::Mouse(mouse_event) => {
                self.send_mouse(mouse_event, root, layout_tree, responses);
            }
        }
    }

    fn send_window<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: WindowEvent,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        for child in root.children.iter_mut() {
            self.send_window(event, child, layout_tree, responses);
        }
        responses.extend(root.window_event(event, layout_tree));
    }

    fn send_mouse<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: MouseEvent,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        match event {
            MouseEvent::Move(position) => {
                self.send_mouse_move(position, root, layout_tree, responses)
            }
            _ => {
                self.send_mouse_inner(event, root, layout_tree, responses);
            }
        }
    }

    fn send_mouse_inner<T: 'static, R: Renderer + 'static>(
        &mut self,
        event: MouseEvent,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        let curr_hover = root.find_element(&mut |el| {
            let clip = &layout_tree[el.layout_key].clip;

            self.curr_mouse_pos.is_some_and(|position| {
                clip.is_inside(position.x.as_float(), position.y.as_float())
            })
        });

        if let Some(curr_hover) = curr_hover
            && let Some(msg) = curr_hover.mouse_event(event, layout_tree).take()
        {
            responses.push(msg)
        }
    }

    fn send_mouse_move<T: 'static, R: Renderer + 'static>(
        &mut self,
        position: LPoint<i32>,
        root: &mut VisualElement<T, R>,
        layout_tree: &mut LayoutElementTree,
        responses: &mut Vec<T>,
    ) {
        let prev_hover = root
            .find_element(&mut |el| {
                let clip = &layout_tree[el.layout_key].clip;

                self.prev_mouse_pos.is_some_and(|position| {
                    clip.is_inside(position.x.as_float(), position.y.as_float())
                })
            })
            .map(|el| el as *mut VisualElement<T, R>);

        let curr_hover = root
            .find_element(&mut |el| {
                layout_tree[el.layout_key]
                    .clip
                    .is_inside(position.x.as_float(), position.y.as_float())
            })
            .map(|el| el as *mut VisualElement<T, R>);

        let (prev_hover, curr_hover) = unsafe {
            (
                prev_hover.map(|prev| &mut *prev),
                curr_hover.map(|curr| &mut *curr),
            )
        };

        let response = match (prev_hover, curr_hover) {
            (None, Some(curr)) => curr.mouse_event(MouseEvent::Enter, layout_tree),
            (Some(prev), None) => prev.mouse_event(MouseEvent::Left, layout_tree),
            (Some(prev), Some(curr)) => {
                if std::ptr::eq(prev, curr) {
                    curr.mouse_event(MouseEvent::Move(self.curr_mouse_pos.unwrap()), layout_tree)
                } else {
                    if let Some(msg) = prev.mouse_event(MouseEvent::Left, layout_tree).take() {
                        responses.push(msg);
                    }

                    curr.mouse_event(MouseEvent::Enter, layout_tree)
                }
            }
            _ => return,
        };

        if let Some(msg) = response.take() {
            responses.push(msg)
        }
    }
}
