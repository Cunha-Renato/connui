use crate::{renderer::RenderCommand, widget::Element};

#[derive(Default)]
pub struct Context {}
impl Context {
    pub fn layout<T>(&mut self, widget: Element<'_, T>) -> Vec<RenderCommand> {
        let node = crate::layout::layout(&widget);

        node.render()
    }
}
