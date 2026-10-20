use connui::{event::*, has_children, has_color, has_style, tree::*};

use crate::{
    style::{StyleRules, StyleSelector},
    *,
};

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
    on_event: Option<EventFn<T>>,
}
impl<T, R: Renderer> Div<T, R> {
    #[inline]
    pub fn on_event(mut self, f: impl Fn(Event) -> Response<T> + 'static) -> Self {
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
        Key::of::<DivElement>()
    }

    #[inline]
    fn mount(self: Box<Self>) -> Element<T, R> {
        Element::new(self.children, DivElement::new(self.style))
    }

    fn update(self: Box<Self>, mut updater: Updater) {
        updater.update(*self, |widget, el: &mut DivElement| {
            el.update(widget.style);
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

pub struct DivElement {
    style: DivStyle,
    state: WidgetStateFlags,
}
impl DivElement {
    fn new(style: StyleSelector<DivStyle>) -> Self {
        let state = WidgetStateFlags::DEFAULT;

        Self {
            style: style.get(state),
            state,
        }
    }

    #[inline]
    fn update(&mut self, style: StyleSelector<DivStyle>) {
        self.style = style.get(self.state);
    }
}
impl<T: 'static, R: Renderer + 'static> ElementSpecs<T, R> for DivElement {
    #[inline]
    fn style(&self) -> &Style {
        &self.style.style
    }

    fn mouse_event(&mut self, event: MouseEvent, _: LayoutContext) -> Response<T> {
        Response::ConsumedEmpty
    }

    fn window_event(&mut self, event: WindowEvent, _: LayoutContext) -> Option<T> {
        None
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
