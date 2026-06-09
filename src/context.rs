use crate::{input::InputEvent, renderer::RenderCommand, widget::Element};

#[derive(Default)]
pub struct Context {
    event_buffer: Vec<InputEvent>,
    dpi: f32,
}
impl Context {
    pub fn dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn layout<T: 'static>(&mut self, widget: Element<T>) -> Vec<RenderCommand> {
        let node = crate::layout::layout(widget, self.dpi);

        node.render()
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.event_buffer.push(event);
    }
}
