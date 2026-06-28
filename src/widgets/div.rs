use super::*;
use crate::{
    event::Event,
    types::Response,
    widget::{Element, Widget},
    widgets::Style,
};

pub struct Div<T: 'static> {
    style: Style,
    children: Vec<Element<T>>,
    on_event: Option<Box<dyn FnMut(&mut Self, Event) -> Response<T>>>,
}
impl<T: 'static> Div<T> {
    #[inline]
    pub fn children(mut self, children: impl Into<Vec<Element<T>>>) -> Self {
        self.children = children.into();
        self
    }

    #[inline]
    pub fn children_iter(
        mut self,
        children: impl IntoIterator<Item = impl Into<Element<T>>>,
    ) -> Self {
        self.children.clear();
        self.extend_children(children);

        self
    }

    #[inline]
    pub fn push_child(&mut self, child: impl Into<Element<T>>) {
        self.children.push(child.into());
    }

    #[inline]
    pub fn extend_children(&mut self, children: impl IntoIterator<Item = impl Into<Element<T>>>) {
        self.children.extend(children.into_iter().map(Into::into));
    }

    #[inline]
    pub fn on_event(mut self, f: impl FnMut(&mut Self, Event) -> Response<T> + 'static) -> Self {
        self.on_event = Some(Box::new(f));
        self
    }
}
impl<T: 'static> Widget<T> for Div<T> {
    #[inline]
    fn get_position(&self) -> Position {
        self.style.position
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.style.size
    }

    #[inline]
    fn get_layout(&self) -> crate::types::Layout {
        self.style.layout
    }

    #[inline]
    fn get_children(&mut self) -> Vec<Element<T>> {
        std::mem::take(&mut self.children)
    }

    fn render(
        &self,
        position: crate::types::Point,
        size: Size,
    ) -> Vec<crate::renderer::RenderCommand> {
        vec![crate::renderer::RenderCommand::DrawRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            color: self.style.color,
        }]
    }

    fn on_event(&mut self, event: crate::event::Event) -> crate::prelude::Response<T> {
        let mut f = std::mem::take(&mut self.on_event);

        let result = if let Some(evfn) = f.as_mut() {
            evfn(self, event)
        } else {
            Response {
                response: None,
                consume: false,
            }
        };

        self.on_event = f;

        result
    }
}
impl<T: 'static> From<Div<T>> for Element<T> {
    #[inline]
    fn from(value: Div<T>) -> Self {
        Self::new(value)
    }
}
impl<T: 'static> Default for Div<T> {
    #[inline]
    fn default() -> Self {
        Self {
            style: Default::default(),
            on_event: Default::default(),
            children: vec![],
        }
    }
}

impl_has_color!(Div<T> { style.color });
impl_has_layout!(Div<T> { style.layout });
impl_has_position!(Div<T> { style.position });
impl_has_margin!(<u16> Div<T> { style.margin });
impl_has_padding!(<u16> Div<T> { style.padding });
impl_has_size!(<crate::types::SizeOp> Div<T> { style.size });
