use crate::{renderer::RenderCommand, widget::Element};

#[derive(Default)]
pub struct Context {
    dpi: f32,
}
impl Context {
    pub fn dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn layout<T>(&mut self, widget: Element<'_, T>) -> Vec<RenderCommand> {
        let node = crate::layout::layout(&widget, self.dpi);

        node.render()
    }
}
