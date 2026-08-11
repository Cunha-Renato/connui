use connui::{prelude::*, renderer::Renderer, widget::Widget};

use crate::Div;

pub struct ScrollDiv<T, R: Renderer> {
    div: Div<T, R>,
}
impl<T, R: Renderer> Widget<T, R> for ScrollDiv<T, R> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.div.get_size()
    }

    #[inline]
    fn get_position(&self) -> Position {
        self.div.get_position()
    }

    #[inline]
    fn get_padding(&self) -> LSides<u16> {
        self.div.get_padding()
    }

    #[inline]
    fn get_margin(&self) -> LSides<u16> {
        self.div.get_margin()
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.div.get_layout()
    }

    #[inline]
    fn get_children(&mut self) -> Vec<Element<T, R>> {
        self.div.get_children()
    }

    fn render(
        &mut self,
        rect: &PRect,
        scissor: &Rect<u32, u32>,
        renderer: &mut R,
        children: &mut [Node<T, R>],
    ) {
        renderer.push_scissor(scissor);
        for child in children {
            child.render(renderer);
        }
        renderer.pop_scissor();
    }

    fn init(&mut self, ctx: &mut connui::state::StateContext<R>) {}

    fn update(
        &mut self,
        rect: LRect<i32, u16>,
        clip: LRect<i32, u16>,
        children: &mut Vec<Node<T, R>>,
        ctx: &mut connui::state::StateContext<R>,
    ) -> bool {
        false
    }

    fn on_event(&mut self, event: connui::event::Event) -> Response<T> {
        Widget::on_event(&mut self.div, event)
    }
}
