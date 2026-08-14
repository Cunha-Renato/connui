use connui::{
    impl_default_widget_layout,
    layout::{LayoutElement, WidgetDesc},
};

use crate::*;

pub struct Div<T, R: Renderer> {
    style: Style,
    on_event: Option<EventFn<T>>,
    children: Children<T, R>,
}
impl<T, R: Renderer> Div<T, R> {
    #[inline]
    pub fn on_event(mut self, f: impl Fn(Event) -> Response<T> + 'static) -> Self {
        self.on_event = Some(f.into());
        self
    }
}
impl<T, R: Renderer> Default for Div<T, R> {
    #[inline]
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_event: Default::default(),
            children: Default::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Div<T, R>> for Element<T, R> {
    fn from(value: Div<T, R>) -> Self {
        Self::new(value)
    }
}

impl<T: 'static, R: Renderer + 'static> WidgetDiff for Div<T, R> {
    fn diff_eq(&self, other: Differ) -> bool {
        other.diff_eq(self, |a, b| {
            a.style.diff(&b.style) && self.children == b.children
        })
    }
}
impl<T, R: Renderer> WidgetDesc for Div<T, R> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.style.size
    }

    #[inline]
    fn get_position(&self) -> Position {
        self.style.position
    }

    #[inline]
    fn get_padding(&self) -> LSides<u16> {
        self.style.padding.clone()
    }

    #[inline]
    fn get_margin(&self) -> LSides<u16> {
        self.style.margin.clone()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.style.layout
    }
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Div<T, R> {
    #[inline]
    fn get_children(&self) -> &[Element<T, R>] {
        &self.children
    }

    #[inline]
    fn get_children_mut(&mut self) -> &mut [Element<T, R>] {
        &mut self.children
    }

    #[inline]
    fn render(
        &mut self,
        render_element: RenderElement,
        layout_element: &LayoutElement,
        renderer: &mut R,
    ) {
        renderer.draw_quad(&render_element.rect, self.style.color, None);
        if !self.children.is_empty() {
            renderer.push_scissor(&render_element.scissor);
            self.children
                .iter_mut()
                .zip(&layout_element.children)
                .for_each(|(child, child_layout)| child.render(child_layout, renderer));
            renderer.pop_scissor();
        }
    }

    fn on_event(&mut self, event: Event) -> Response<T> {
        if let Some(evfn) = self.on_event.as_ref() {
            evfn(event)
        } else {
            Response {
                response: None,
                consume: false,
            }
        }
    }
}

impl_has_style!({T, R: Renderer} trait for Div {T, R} with { style });
impl_has_children!({T, R: Renderer} trait {T, R} for Div{T, R} with { children });

impl_default_widget_layout!({T: 'static, R: Renderer + 'static} Div { T, R });
