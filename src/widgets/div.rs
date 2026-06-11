use super::*;
use crate::{
    widget::{Element, Widget},
    widgets::Style,
};

#[derive(Default)]
pub struct Div<T: 'static> {
    style: Style,
    children: Vec<Element<T>>,
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
}

impl_has_color!(Div<T> { style.color });
impl_has_layout!(Div<T> { style.layout });
impl_has_position!(Div<T> { style.position });
impl_has_margin!(<u16> Div<T> { style.margin });
impl_has_padding!(<u16> Div<T> { style.padding });
impl_has_size!(<crate::types::SizeOp> Div<T> { style.size });
