use bitflags::bitflags;

use crate::{renderer::Renderer, tree::VisualElement, types::*};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WindowEvent {
    CursorEnter,
    CursorLeft,
}

pub mod external {
    use crate::types::*;

    use super::{MouseButton, WindowEvent};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum InputEvent {
        Window(WindowEvent),
        Mouse(MouseInputEvent),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum MouseInputEvent {
        Button { button: MouseButton, pressed: bool },
        Move(LPosition<i32>),
        Scroll(LPosition<i32>),
    }
}

pub mod internal {
    use bitflags::bitflags;

    use super::{MouseButton, WindowEvent};
    use crate::types::*;

    #[derive(Debug, Clone, Copy)]
    pub enum Event {
        Window(WindowEvent),
        Mouse(MouseEvent),
    }
    impl Event {
        pub const fn kind(&self) -> EventKind {
            match self {
                Event::Mouse(mouse_event) => match mouse_event.event {
                    MouseEventKind::Enter | MouseEventKind::Left | MouseEventKind::Move => {
                        EventKind::MOVE
                    }
                    MouseEventKind::Button { .. } | MouseEventKind::Click { .. } => {
                        EventKind::BUTTON
                    }
                    MouseEventKind::Scroll(_) => EventKind::SCROLL,
                },
                _ => EventKind::empty(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum EventTarget {
        This,
        Child,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct MouseEvent {
        pub event: MouseEventKind,
        pub position: LPosition<i32>,
        pub target: EventTarget,
    }
    impl MouseEvent {
        #[inline]
        pub fn new(event: MouseEventKind, position: LPosition<i32>) -> Self {
            Self {
                event,
                position,
                target: EventTarget::This,
            }
        }

        pub(super) fn this(mut self) -> Self {
            self.target = EventTarget::This;
            self
        }

        pub(super) fn child(mut self) -> Self {
            self.target = EventTarget::Child;
            self
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum MouseEventKind {
        Enter,
        Left,
        Move,
        Scroll(LPosition<i32>),
        Click { button: MouseButton, count: u8 },
        Button { button: MouseButton, pressed: bool },
    }

    bitflags! {
        #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct EventKind: u8 {
            const MOVE = 0b1;
            const BUTTON = 0b10;
            const SCROLL = 0b100;
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    pub(super) struct ClickContext([ClickData; MouseButton::ALL.len()]);
    impl ClickContext {
        // Assuming only one button.
        pub fn click(&mut self, button: MouseButton) -> u8 {
            if button.bits() == 0 {
                return 0;
            }

            self.0[button.bits().ilog10() as usize].click()
        }
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct ClickData {
        last_click: Option<std::time::Instant>,
        count: u8,
    }
    impl ClickData {
        const INCREMENT_THRESHOLD: std::time::Duration = std::time::Duration::from_millis(500);

        fn click(&mut self) -> u8 {
            let elapsed = self
                .last_click
                .map_or(Self::INCREMENT_THRESHOLD, |instant| instant.elapsed());
            self.last_click = Some(std::time::Instant::now());

            if elapsed < Self::INCREMENT_THRESHOLD {
                self.count += 1;
            } else {
                self.count = 1;
            }

            self.count
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct MousePositionContext {
        pub previous: LPosition<i32>,
        pub current: LPosition<i32>,
    }
    impl MousePositionContext {
        pub fn new_position(&mut self, new_position: LPosition<i32>) {
            std::mem::swap(&mut self.current, &mut self.previous);
            self.current = new_position;
        }
    }
}

#[derive(Default)]
pub struct InputContext {
    external_buffer: Vec<external::InputEvent>,
    position_context: internal::MousePositionContext,
    click_context: internal::ClickContext,
}
impl InputContext {
    #[inline]
    pub(crate) fn stage_external_event(&mut self, event: external::InputEvent) {
        self.external_buffer.push(event);
    }

    pub(crate) fn process_events<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
    ) -> Vec<T> {
        let mut messages = vec![];

        self.events(|this, event| match event {
            internal::Event::Window(window_event) => {
                send_events_all(window_event, root, &mut messages, &mut |element, event| {
                    element.window_event(event)
                })
            }
            internal::Event::Mouse(mouse_event) => {
                this.process_mouse(root, mouse_event, &mut messages);
            }
        });

        messages
    }

    fn events<F: FnMut(&mut Self, internal::Event)>(&mut self, mut handler: F) {
        use internal::*;

        for external_event in std::mem::take(&mut self.external_buffer) {
            let current_position = self.position_context.current;

            match external_event {
                external::InputEvent::Window(window_event) => {
                    handler(self, Event::Window(window_event))
                }
                external::InputEvent::Mouse(mouse_input_event) => match mouse_input_event {
                    external::MouseInputEvent::Button { button, pressed } => {
                        handler(
                            self,
                            Event::Mouse(MouseEvent::new(
                                MouseEventKind::Button { button, pressed },
                                current_position,
                            )),
                        );

                        if !pressed {
                            let count = self.click_context.click(button);

                            handler(
                                self,
                                Event::Mouse(MouseEvent::new(
                                    MouseEventKind::Click { button, count },
                                    current_position,
                                )),
                            )
                        }
                    }
                    external::MouseInputEvent::Move(position) => {
                        self.position_context.new_position(position);

                        handler(
                            self,
                            Event::Mouse(MouseEvent::new(MouseEventKind::Move, position)),
                        );
                    }
                    external::MouseInputEvent::Scroll(delta) => handler(
                        self,
                        Event::Mouse(MouseEvent::new(
                            MouseEventKind::Scroll(delta),
                            current_position,
                        )),
                    ),
                },
            }
        }
    }

    // TODO: Implement capture.
    fn process_mouse<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        event: internal::MouseEvent,
        messages: &mut Vec<T>,
    ) {
        match event.event {
            internal::MouseEventKind::Scroll(_)
            | internal::MouseEventKind::Click { .. }
            | internal::MouseEventKind::Button { .. } => {
                if let Some(current_hover) = find_element_ptr(root, self.position_context.current) {
                    messages.extend(unsafe { &mut *current_hover }.mouse_event(event).take());

                    send_events_bubble(
                        event.child(),
                        unsafe { &mut *current_hover },
                        messages,
                        &mut |element, event| element.mouse_event(event).take(),
                    )
                }
            }
            internal::MouseEventKind::Move => {
                let previous_hover = find_element_ptr(root, self.position_context.previous);
                let current_hover = find_element_ptr(root, self.position_context.current);

                let mut send_event =
                    |element: &mut VisualElement<T, R>, mouse_event: internal::MouseEvent| {
                        element.mouse_event(mouse_event.this());

                        send_events_bubble(
                            mouse_event,
                            element,
                            messages,
                            &mut |element, event| element.mouse_event(event.child()).take(),
                        );
                    };

                match (previous_hover, current_hover) {
                    (Some(previous_hover), None) => {
                        let previous_hover = unsafe { &mut *previous_hover };

                        let mouse_event = internal::MouseEvent::new(
                            internal::MouseEventKind::Left,
                            self.position_context.current,
                        );

                        send_event(previous_hover, mouse_event);
                    }
                    (None, Some(current_hover)) => {
                        let current_hover = unsafe { &mut *current_hover };

                        let mouse_event = internal::MouseEvent::new(
                            internal::MouseEventKind::Enter,
                            self.position_context.current,
                        );

                        send_event(current_hover, mouse_event);
                    }
                    (Some(previous_hover), Some(current_hover))
                        if !std::ptr::eq(previous_hover, current_hover) =>
                    {
                        let mouse_event_left = internal::MouseEvent::new(
                            internal::MouseEventKind::Left,
                            self.position_context.current,
                        );
                        let mouse_event_enter = internal::MouseEvent::new(
                            internal::MouseEventKind::Enter,
                            self.position_context.current,
                        );

                        unsafe {
                            // Preventing to send Left if the cursor is still inside.
                            if !position_filter(&mut *previous_hover, self.position_context.current)
                            {
                                send_event(&mut *previous_hover, mouse_event_left);
                            }
                        }

                        unsafe {
                            // Preventing to send Enter if the cursor is still inside.
                            if !position_filter(&mut *current_hover, self.position_context.previous)
                            {
                                send_event(&mut *current_hover, mouse_event_enter);
                            }
                        }
                    }
                    _ => {}
                }

                // Move event.
                if let Some(current_hover) = current_hover {
                    messages.extend(unsafe { &mut *current_hover }.mouse_event(event).take())
                }
            }
            _ => {}
        }
    }
}

/// Sends event from top to bottom.
fn send_events_all<E, ES, T, R>(
    event: E,
    element: &mut VisualElement<T, R>,
    messages: &mut Vec<T>,
    event_send: &mut ES,
) where
    T: 'static,
    E: Copy,
    R: Renderer + 'static,
    ES: FnMut(&mut VisualElement<T, R>, E) -> Option<T>,
{
    messages.extend(event_send(element, event));

    for child in &mut element.children {
        send_events_all(event, child, messages, event_send);
    }
}

/// Sends event from bottom to top.
fn send_events_bubble<E, ES, T, R>(
    event: E,
    element: &mut VisualElement<T, R>,
    messages: &mut Vec<T>,
    event_send: &mut ES,
) where
    T: 'static,
    E: Copy,
    R: Renderer + 'static,
    ES: FnMut(&mut VisualElement<T, R>, E) -> Option<T>,
{
    if let Some(parent_ptr) = element.parent {
        let parent = unsafe { &mut *parent_ptr.as_ptr() };

        messages.extend(event_send(parent, event));
        send_events_bubble(event, parent, messages, event_send);
    }
}

fn position_filter<T: 'static, R: Renderer + 'static>(
    element: &mut VisualElement<T, R>,
    position: LPosition<i32>,
) -> bool {
    element
        .layout_element
        .borrow()
        .clip
        .contains(position.x.as_float(), position.y.as_float())
}

fn find_element_ptr<'a, T: 'static, R: Renderer + 'static>(
    root: &mut VisualElement<'a, T, R>,
    position: LPosition<i32>,
) -> Option<*mut VisualElement<'a, T, R>> {
    root.find_element(&mut |element| position_filter(element, position))
        .map(|element| element as *mut _)
}

// impl InputContext {
//     pub(crate) fn send_input<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         root: &mut VisualElement<T, R>,
//     ) -> Vec<T> {
//         if self.buffer.is_empty() {
//             return vec![];
//         }

//         let mut responses = vec![];

//         for event in std::mem::take(&mut self.buffer) {
//             let captured = self.send_capture(event, root, &mut responses);

//             if !captured {
//                 self.send_all(event, root, &mut responses);
//             }
//         }

//         self.next_frame();
//         responses
//     }

//     fn send_capture<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         event: Event,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) -> bool {
//         let (Some(element), Event::Mouse(mouse_event)) = (
//             root.find_element(&mut |el| event.kind().intersects(el.element.input_capture())),
//             event,
//         ) else {
//             return false;
//         };

//         let element_ptr = element as *mut VisualElement<T, R>;
//         let prev_capture = element.element.input_capture();

//         let response = element.mouse_event(mouse_event);
//         let curr_capture = element.element.input_capture();
//         let consumed = response.consumed();
//         responses.extend(response.take());

//         let lost_move_capture = consumed
//             && prev_capture.contains(EventKind::MOVE)
//             && !curr_capture.contains(EventKind::MOVE);

//         if lost_move_capture {
//             self.reconcile_hover_after_capture_loss(root, element_ptr, responses);
//         }

//         consumed
//     }

//     fn reconcile_hover_after_capture_loss<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         root: &mut VisualElement<T, R>,
//         prev_hover: *mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         let curr_hover = root
//             .find_element(&mut |el| {
//                 let clip = &&el.layout_element.borrow().clip;

//                 self.curr_mouse_pos.is_some_and(|position| {
//                     clip.contains(position.x.as_float(), position.y.as_float())
//                 })
//             })
//             .map(|el| el as *mut VisualElement<T, R>);

//         if curr_hover.is_some_and(|c| std::ptr::eq(c, prev_hover)) {
//             return; // still the same element — nothing changed
//         }

//         unsafe {
//             responses.extend((*prev_hover).mouse_event(MouseEvent::Left).take());
//             responses
//                 .extend(curr_hover.and_then(|curr| (*curr).mouse_event(MouseEvent::Enter).take()));
//         }
//     }

//     fn send_all<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         event: Event,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         match event {
//             Event::Window(window_event) => {
//                 self.send_window(window_event, root, responses);
//             }
//             Event::Mouse(mouse_event) => {
//                 self.send_mouse(mouse_event, root, responses);
//             }
//         }
//     }

//     fn send_window<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         event: WindowEvent,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         for child in root.children.iter_mut() {
//             self.send_window(event, child, responses);
//         }
//         responses.extend(root.window_event(event));
//     }

//     fn send_mouse<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         event: MouseEvent,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         match event {
//             MouseEvent::Move(position) => self.send_mouse_move(position, root, responses),
//             _ => {
//                 self.send_mouse_inner(event, root, responses);
//             }
//         }
//     }

//     fn send_mouse_inner<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         event: MouseEvent,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         if let Some(position) = self.curr_mouse_pos {
//             let curr_hover = root.find_element(&mut |el| {
//                 el.layout_element
//                     .borrow()
//                     .clip
//                     .contains(position.x.as_float(), position.y.as_float())
//             });

//             if let Some(curr_hover) = curr_hover
//                 && let Some(msg) = curr_hover.mouse_event(event).take()
//             {
//                 responses.push(msg)
//             }
//         }
//     }

//     fn send_mouse_move<T: 'static, R: Renderer + 'static>(
//         &mut self,
//         position: LPosition<i32>,
//         root: &mut VisualElement<T, R>,
//         responses: &mut Vec<T>,
//     ) {
//         let prev_hover = root
//             .find_element(&mut |el| {
//                 let clip = &el.layout_element.borrow().clip;

//                 self.prev_mouse_pos.is_some_and(|position| {
//                     clip.contains(position.x.as_float(), position.y.as_float())
//                 })
//             })
//             .map(|el| el as *mut VisualElement<T, R>);

//         let curr_hover = root
//             .find_element(&mut |el| {
//                 el.layout_element
//                     .borrow()
//                     .clip
//                     .contains(position.x.as_float(), position.y.as_float())
//             })
//             .map(|el| el as *mut VisualElement<T, R>);

//         let (prev_hover, curr_hover) = unsafe {
//             (
//                 prev_hover.map(|prev| &mut *prev),
//                 curr_hover.map(|curr| &mut *curr),
//             )
//         };

//         let response = match (prev_hover, curr_hover) {
//             (None, Some(curr)) => curr.mouse_event(MouseEvent::Enter),
//             (Some(prev), None) => prev.mouse_event(MouseEvent::Left),
//             (Some(prev), Some(curr)) => {
//                 if std::ptr::eq(prev, curr) {
//                     curr.mouse_event(MouseEvent::Move(self.curr_mouse_pos.unwrap()))
//                 } else {
//                     if let Some(msg) = prev.mouse_event(MouseEvent::Left).take() {
//                         responses.push(msg);
//                     }

//                     curr.mouse_event(MouseEvent::Enter)
//                 }
//             }
//             _ => return,
//         };

//         if let Some(msg) = response.take() {
//             responses.push(msg)
//         }
//     }
// }
