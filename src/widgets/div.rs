use super::*;
use crate::prelude::*;
use crate::renderer::RenderCommand;

pub struct Div<T: 'static> {
    style: Style,
    on_event: Option<EventFn<T>>,
    children: Children<T>,
}
impl<T: 'static> Default for Div<T> {
    fn default() -> Self {
        Self {
            style: Style::default(),
            on_event: None,
            children: Children::default(),
        }
    }
}
impl<T: 'static> From<Div<T>> for Element<T> {
    fn from(value: Div<T>) -> Self {
        Self::new(value)
    }
}
impl<T: 'static> Widget<T> for Div<T> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.style.size
    }

    #[inline]
    fn get_position(&self) -> Position {
        self.style.position
    }

    #[inline]
    fn get_padding(&self) -> Sides<u16> {
        self.style.padding
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        self.style.margin
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.style.layout
    }

    #[inline]
    fn get_children(&mut self) -> Vec<Element<T>> {
        match std::mem::take(&mut self.children) {
            Some(vec) => *vec,
            None => vec![],
        }
    }

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand> {
        vec![RenderCommand::DrawRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            color: self.style.color,
        }]
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
impl<T: 'static> Div<T> {
    #[inline]
    pub fn on_event(mut self, f: impl Fn(Event) -> Response<T> + 'static) -> Self {
        self.on_event = Some(f.into());
        self
    }
}

impl_has_style!(Div<T> { style });
impl_has_children!(<T> Div<T> { children });
