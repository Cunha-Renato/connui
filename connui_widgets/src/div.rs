use connui::{event::EventKind, tree::*};

use crate::*;

pub struct Div<T, R: Renderer> {
    style: Style,
    children: Vec<widget::Widget<T, R>>,
    on_event: Option<EventFn<T>>,
    color: Color,
}
impl<T, R: Renderer> Div<T, R> {
    #[inline]
    pub fn on_event(mut self, f: impl Fn(Event) -> Response<T> + 'static) -> Self {
        self.on_event = Some(f.into());
        self
    }
}
impl<T: 'static, R: Renderer + 'static> WidgetSpecs<T, R> for Div<T, R> {
    #[inline]
    fn key(&self) -> Key {
        Key::of::<DivElement>()
    }

    #[inline]
    fn mount(self: Box<Self>) -> Element<T, R> {
        Element::new(
            self.children,
            DivElement {
                style: self.style,
                color: self.color,
                capture: EventKind::empty(),
                hover: false,
            },
        )
    }

    fn update(self: Box<Self>, updater: Updater) {
        updater.update(*self, |widget, el: &mut DivElement| {
            el.style = widget.style;
            el.color = widget.color;
        });
    }

    fn children(&mut self) -> Vec<widget::Widget<T, R>> {
        std::mem::take(&mut self.children)
    }
}
impl<T, R: Renderer> Default for Div<T, R> {
    #[inline]
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_event: Default::default(),
            children: Default::default(),
            color: Default::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Div<T, R>> for Widget<T, R> {
    #[inline]
    fn from(value: Div<T, R>) -> Self {
        Self::new(value)
    }
}

pub struct DivElement {
    style: Style,
    color: Color,
    capture: EventKind,
    hover: bool,
}
impl<T: 'static, R: Renderer + 'static> ElementSpecs<T, R> for DivElement {
    #[inline]
    fn style(&self) -> &Style {
        &self.style
    }

    fn input_event(&mut self, event: Event, layout_element: &LayoutElement) -> Response<T> {
        use connui::event::*;
        let mut should_consume = false;

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                MouseEvent::LeftWindow => self.hover = false,
                MouseEvent::Press(_) => {
                    if self.hover {
                        self.capture = EventKind::BUTTON | EventKind::MOVE;

                        should_consume = true;
                    }
                }
                MouseEvent::Release(_) => {
                    self.capture = EventKind::empty();

                    should_consume = true;
                }
                MouseEvent::Move(point) => {
                    self.hover = layout_element
                        .rect
                        .is_inside(point.x.as_float(), point.y.as_float());

                    should_consume = self.hover;
                }
                _ => {}
            },
        }

        if should_consume {
            Response::ConsumedEmpty
        } else {
            Response::None
        }
    }

    #[inline]
    fn input_consumed(&mut self, kind: EventKind) {
        if kind.contains(EventKind::MOVE) {
            self.hover = false;
        }
    }

    #[inline]
    fn input_capture(&self) -> EventKind {
        self.capture
    }

    fn render(
        &self,
        children: &[VisualElement<T, R>],
        render_element: RenderElement,
        layout_tree: &layout::LayoutElementTree,
        renderer: &mut R,
    ) {
        let color = if self.hover {
            Color::MAGENTA
        } else {
            self.color
        };
        renderer.draw_quad(&render_element.rect, color, None);

        if !children.is_empty() && render_element.can_render_children() {
            renderer.push_scissor(&render_element.scissor);
            for child in children {
                child.render(layout_tree, renderer);
            }
            renderer.pop_scissor();
        }
    }
}

impl_has_style!({T, R: Renderer} trait for Div {T, R} with { style });
impl_has_color!({T, R: Renderer} trait for Div {T, R} with { color });
impl_has_children!({T, R: Renderer} trait {T, R} for Div{T, R} with { children });
