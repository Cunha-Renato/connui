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
    Focused(bool),
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
                Event::Mouse(mouse_event) => mouse_event.kind(),
                Event::Window(_) => EventKind::empty(),
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
        pub position: Option<LPosition<i32>>,
        pub target: EventTarget,
    }
    impl MouseEvent {
        #[inline]
        pub fn new(event: MouseEventKind, position: Option<LPosition<i32>>) -> Self {
            Self {
                event,
                position,
                target: EventTarget::This,
            }
        }

        pub const fn kind(&self) -> EventKind {
            self.event.kind()
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
        Click {
            button: MouseButton,
            count: u8,
        },
        Button {
            button: MouseButton,
            state: ButtonState,
        },
    }
    impl MouseEventKind {
        pub const fn kind(&self) -> EventKind {
            match self {
                MouseEventKind::Enter | MouseEventKind::Left | MouseEventKind::Move => {
                    EventKind::MOVE
                }
                MouseEventKind::Scroll(_) => EventKind::SCROLL,
                MouseEventKind::Click { .. } | MouseEventKind::Button { .. } => EventKind::BUTTON,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum ButtonState {
        Pressed(bool),
        /// Fired when the window was minimized or lost focus.
        /// Meaning that the button state is no longer
        /// the application responsability.
        Canceled,
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
        pub fn click(&mut self, button: MouseButton) -> Vec<(MouseButton, u8)> {
            button
                .iter()
                .map(move |b| (b, self.0[button.bits().ilog10() as usize].click()))
                .collect()
        }

        pub fn clear(&mut self) {
            for click_data in &mut self.0 {
                click_data.clear();
            }
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

        fn clear(&mut self) {
            self.last_click = None;
            self.count = 0;
        }
    }

    #[derive(Debug, Default)]
    pub(super) struct MousePositionContext {
        pub previous: Option<LPosition<i32>>,
        pub current: Option<LPosition<i32>>,
    }
    impl MousePositionContext {
        pub const fn new_position(&mut self, new_position: Option<LPosition<i32>>) {
            self.previous = self.current;
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
    pub(crate) fn minimize<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
    ) {
        self.click_context.clear();
        self.fire_events(root);
    }

    #[inline]
    pub(crate) fn stage_external_event(&mut self, event: external::InputEvent) {
        self.external_buffer.push(event);
    }

    /// Fires [`Event`](internal::Event)s to the relevant elements.
    pub(crate) fn fire_events<T: 'static, R: Renderer + 'static>(
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
                if !this.process_capture(root, mouse_event, &mut messages) {
                    this.process_mouse(root, mouse_event, &mut messages);
                }
            }
        });

        messages
    }

    /// Returns true if an element is holding capture of the event kind.
    fn process_capture<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut VisualElement<T, R>,
        event: internal::MouseEvent,
        messages: &mut Vec<T>,
    ) -> bool {
        let event_kind = event.kind();

        let Some(element) =
            root.find_element(&mut |element| element.element.input_capture().contains(event_kind))
        else {
            return false;
        };

        let held_cursor = element
            .element
            .input_capture()
            .contains(internal::EventKind::MOVE);

        messages.extend(element.mouse_event(event).message);

        // This runs when pointer movement was captured and released.
        // Fires a synthetic move event simulating the cursor jumping from the
        // origin of the element to the current mouse position.
        if held_cursor
            && !element
                .element
                .input_capture()
                .contains(internal::EventKind::MOVE)
        {
            self.position_context.previous = Some(
                element
                    .layout_element
                    .borrow()
                    .clip
                    .position
                    .map(|p| p.as_signed()),
            );

            self.process_mouse(
                root,
                internal::MouseEvent::new(
                    internal::MouseEventKind::Move,
                    self.position_context.current,
                ),
                messages,
            );
        }

        true
    }

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
                if let Some(current_hover) =
                    find_element_position_inside(root, self.position_context.current)
                {
                    send_events_bubble_with_this(
                        event,
                        unsafe { &mut *current_hover },
                        messages,
                        &mut |this, event| this.mouse_event(event.this()),
                        &mut |parent, event| parent.mouse_event(event.child()),
                    )
                }
            }
            internal::MouseEventKind::Move => {
                let previous_hover =
                    find_element_position_inside(root, self.position_context.previous);
                let current_hover =
                    find_element_position_inside(root, self.position_context.current);

                let mut send_event =
                    |element: &mut VisualElement<T, R>, mouse_event: internal::MouseEvent| {
                        send_events_bubble_with_this(
                            mouse_event,
                            element,
                            messages,
                            &mut |this, event| this.mouse_event(event.this()),
                            &mut |parent, event| parent.mouse_event(event.child()),
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
                    messages.extend(
                        unsafe { &mut *current_hover }
                            .mouse_event(event.this())
                            .message,
                    )
                }
            }
            _ => {}
        }
    }

    fn events<F: FnMut(&mut Self, internal::Event)>(&mut self, mut handler: F) {
        use internal::*;

        for external_event in std::mem::take(&mut self.external_buffer) {
            match external_event {
                external::InputEvent::Window(window_event) => {
                    match window_event {
                        WindowEvent::CursorLeft => {
                            self.position_context.new_position(None);

                            handler(
                                self,
                                Event::Mouse(MouseEvent::new(MouseEventKind::Move, None)),
                            );
                        }
                        WindowEvent::Focused(false) => {
                            handler(
                                self,
                                Event::Mouse(MouseEvent::new(
                                    MouseEventKind::Button {
                                        button: MouseButton::all(),
                                        state: ButtonState::Canceled,
                                    },
                                    None,
                                )),
                            );
                        }
                        _ => {}
                    }

                    handler(self, Event::Window(window_event));
                }
                external::InputEvent::Mouse(mouse_input_event) => match mouse_input_event {
                    external::MouseInputEvent::Button { button, pressed } => {
                        handler(
                            self,
                            Event::Mouse(MouseEvent::new(
                                MouseEventKind::Button {
                                    button,
                                    state: ButtonState::Pressed(pressed),
                                },
                                self.position_context.current,
                            )),
                        );

                        if !pressed {
                            for (button, count) in self.click_context.click(button) {
                                handler(
                                    self,
                                    Event::Mouse(MouseEvent::new(
                                        MouseEventKind::Click { button, count },
                                        self.position_context.current,
                                    )),
                                )
                            }
                        }
                    }
                    external::MouseInputEvent::Move(position) => {
                        self.position_context.new_position(Some(position));

                        handler(
                            self,
                            Event::Mouse(MouseEvent::new(MouseEventKind::Move, Some(position))),
                        );
                    }
                    external::MouseInputEvent::Scroll(delta) => {
                        handler(
                            self,
                            Event::Mouse(MouseEvent::new(
                                MouseEventKind::Scroll(delta),
                                self.position_context.current,
                            )),
                        );
                    }
                },
            }
        }
    }
}

/// Sends [`Event`](internal::Event) from top to bottom.
///
/// Will send to the leaf nodes first all the way up to the provided element.
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

/// Sends [`Event`](internal::Event) from bottom to top, starting with the provided element.
fn send_events_bubble_with_this<E, TS, PS, T, R>(
    event: E,
    element: &mut VisualElement<T, R>,
    messages: &mut Vec<T>,
    this_send: &mut TS,
    parent_send: &mut PS,
) where
    T: 'static,
    E: Copy,
    R: Renderer + 'static,
    TS: FnMut(&mut VisualElement<T, R>, E) -> Response<T>,
    PS: FnMut(&mut VisualElement<T, R>, E) -> Response<T>,
{
    let response = this_send(element, event);
    messages.extend(response.message);
    if response.bubble {
        send_events_bubble(event, element, messages, parent_send);
    }
}
/// Sends [`Event`](internal::Event) from bottom to top, excluding the provided element.
fn send_events_bubble<E, ES, T, R>(
    event: E,
    element: &mut VisualElement<T, R>,
    messages: &mut Vec<T>,
    event_send: &mut ES,
) where
    T: 'static,
    E: Copy,
    R: Renderer + 'static,
    ES: FnMut(&mut VisualElement<T, R>, E) -> Response<T>,
{
    if let Some(parent_ptr) = element.parent {
        let parent = unsafe { &mut *parent_ptr.as_ptr() };

        let response = event_send(parent, event);
        messages.extend(response.message);

        if response.bubble {
            send_events_bubble(event, parent, messages, event_send);
        }
    }
}

/// Helper function that checks if the position is inside the element's clip.
fn position_filter<T: 'static, R: Renderer + 'static>(
    element: &mut VisualElement<T, R>,
    position: Option<LPosition<i32>>,
) -> bool {
    position.is_some_and(|position| {
        element
            .layout_element
            .borrow()
            .clip
            .contains(position.x.as_float(), position.y.as_float())
    })
}

/// Helper function that returns the pointer to an element which the position is inside.
fn find_element_position_inside<'a, T: 'static, R: Renderer + 'static>(
    root: &mut VisualElement<'a, T, R>,
    position: Option<LPosition<i32>>,
) -> Option<*mut VisualElement<'a, T, R>> {
    root.find_element(&mut |element| position_filter(element, position))
        .map(|element| element as *mut _)
}
