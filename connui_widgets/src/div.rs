use connui::{event::*, has_children, has_color, has_style, tree::*};

use crate::*;

pub struct Div<T, R: Renderer> {
    name: String,
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

    #[inline]
    pub fn name(mut self, name: String) -> Self {
        self.name = name;
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
                name: self.name,
                style: self.style,
                color: self.color,
                capture: EventKind::empty(),
                hover: false,
            },
        )
    }

    fn update(self: Box<Self>, mut updater: Updater) {
        updater.update(*self, |widget, el: &mut DivElement| {
            el.name = widget.name;
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
            name: "UNKNOWN".to_string(),
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

has_style!({T, R: Renderer} Div {T, R} with {style});
has_color!({T, R: Renderer} Div {T, R} with {color});
has_children!({T, R: Renderer} Div {T, R} => {T, R} with {children});

pub struct DivElement {
    name: String,
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

    fn mouse_event(&mut self, event: MouseEvent, _: LayoutContext) -> Response<T> {
        match event {
            MouseEvent::Enter => self.hover = true,
            MouseEvent::Left => self.hover = false,
            MouseEvent::Press(MouseButton::LEFT) => {
                self.capture = EventKind::BUTTON | EventKind::MOVE;
            }
            MouseEvent::Release(MouseButton::LEFT) => {
                self.capture = EventKind::empty();
            }
            _ => {}
        }

        Response::ConsumedEmpty
    }

    fn window_event(&mut self, event: WindowEvent, _: LayoutContext) -> Option<T> {
        if let WindowEvent::CurserLeft = event {
            self.hover = false;
        }

        None
    }

    #[inline]
    fn input_capture(&self) -> EventKind {
        self.capture
    }

    fn render(
        &self,
        context: LayoutContextRef,
        children: &[VisualElement<T, R>],
        render_element: RenderElement,
        renderer: &mut R,
    ) {
        let mut color = self.color;

        if self.hover {
            color = Color::MAGENTA;
        }

        if !self.capture.is_empty() {
            color = Color::CYAN;
        }

        renderer.draw_quad(&render_element.rect, color, None);

        if !children.is_empty() && render_element.can_render_children() {
            renderer.push_scissor(&render_element.scissor);
            for child in children {
                child.render(context, renderer);
            }
            renderer.pop_scissor();
        }
    }
}
