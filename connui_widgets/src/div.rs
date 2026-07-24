use crate::*;

pub struct Div<T: 'static, R: Renderer> {
    style: Style,
    on_event: Option<EventFn<T>>,
    children: Children<T, R>,
}
impl<T: 'static, R: Renderer> Div<T, R> {
    #[inline]
    pub fn on_event(mut self, f: impl Fn(Event) -> Response<T> + 'static) -> Self {
        self.on_event = Some(f.into());
        self
    }
}
impl<T: 'static, R: Renderer> Default for Div<T, R> {
    fn default() -> Self {
        Self {
            style: Style::default(),
            on_event: None,
            children: Children::default(),
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Div<T, R>> for Element<T, R> {
    fn from(value: Div<T, R>) -> Self {
        Self::new(value)
    }
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Div<T, R> {
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
    fn get_children(&mut self) -> Vec<Element<T, R>> {
        std::mem::take(&mut self.children)
    }

    #[inline]
    fn render(&self, rect: Rect, scissor: Rect, renderer: &mut R, children: &[Node<T, R>]) {
        renderer.draw_quad(rect, self.style.color, None);
        if !children.is_empty() {
            renderer.push_scissor(scissor);
            children.iter().for_each(|c| c.render(renderer));
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
