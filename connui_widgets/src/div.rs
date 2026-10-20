use crate::{
    style::{StyleRules, StyleSelector},
    *,
};
use connui::{event::*, has_children, has_color, has_style, tree::*};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DivStyle {
    pub style: Style,
    pub color: Color,
}
has_style!(DivStyle with {style});
has_color!(DivStyle with {color});

pub struct Div<T, R: Renderer> {
    style: StyleSelector<'static, DivStyle>,
    children: Vec<widget::Widget<T, R>>,
    on_event: Option<Callback<Event, T>>,
}
impl<T, R: Renderer> Div<T, R> {
    #[inline]
    pub fn on_event(mut self, f: impl Into<Callback<Event, T>>) -> Self {
        self.on_event = Some(f.into());
        self
    }

    pub fn style(mut self, style: StyleSelector<'static, DivStyle>) -> Self {
        self.style = style;
        self
    }

    pub fn style_base(mut self, base: DivStyle) -> Self {
        self.style.base = base;
        self
    }

    pub fn style_rules(mut self, rules: StyleRules<'static, DivStyle>) -> Self {
        self.style.rules = rules;
        self
    }
}
impl<T: 'static, R: Renderer + 'static> WidgetSpecs<T, R> for Div<T, R> {
    #[inline]
    fn key(&self) -> Key {
        Key::of::<DivElement<T>>()
    }

    #[inline]
    fn mount(self: Box<Self>) -> Element<T, R> {
        Element::new(self.children, DivElement::new(self.style, self.on_event))
    }

    fn update(self: Box<Self>, mut updater: Updater) {
        updater.update(*self, |widget, el: &mut DivElement<T>| {
            el.update(widget.style, widget.on_event);
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
            style: crate::style::default::div::STYLE_SELECTOR,
            on_event: Default::default(),
            children: Default::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Div<T, R>> for Widget<T, R> {
    #[inline]
    fn from(value: Div<T, R>) -> Self {
        Self::new(value)
    }
}

has_children!({T, R: Renderer} Div {T, R} => {T, R} with {children});

pub struct DivElement<T> {
    style: DivStyle,
    on_event: Option<Callback<Event, T>>,
    state: WidgetStateFlags,
}
impl<T> DivElement<T> {
    fn new(style: StyleSelector<DivStyle>, on_event: Option<Callback<Event, T>>) -> Self {
        let state = WidgetStateFlags::DEFAULT;

        Self {
            style: style.get(state),
            on_event,
            state,
        }
    }

    #[inline]
    fn update(&mut self, style: StyleSelector<DivStyle>, on_event: Option<Callback<Event, T>>) {
        self.style = style.get(self.state);
        self.on_event = on_event;
    }
}
impl<T: 'static, R: Renderer + 'static> ElementSpecs<T, R> for DivElement<T> {
    #[inline]
    fn style(&self) -> &Style {
        &self.style.style
    }

    fn mouse_event(&mut self, event: MouseEvent, _: LayoutContext) -> Response<T> {
        match event {
            MouseEvent::Enter => self.state.set_hover(true),
            MouseEvent::Left => self.state.set_hover(false),
            _ => {}
        }

        if let Some(msg) = self.on_event.as_ref().map(|f| f.call(Event::Mouse(event))) {
            Response::Consumed(msg)
        } else {
            Response::ConsumedEmpty
        }
    }

    fn window_event(&mut self, event: WindowEvent, _: LayoutContext) -> Option<T> {
        self.on_event.as_ref().map(|f| f.call(Event::Window(event)))
    }

    #[inline]
    fn input_capture(&self) -> EventKind {
        EventKind::empty()
    }

    fn render(
        &self,
        context: LayoutContextRef,
        children: &[VisualElement<T, R>],
        render_element: RenderElement,
        renderer: &mut R,
    ) {
        renderer.draw_quad(&render_element.rect, self.style.color, None);
        if !children.is_empty() {
            renderer.push_scissor(&render_element.scissor);
            for child in children {
                child.render(context, renderer);
            }
            renderer.pop_scissor();
        }
    }
}
