use crate::{
    input::{InputEvent, InputState},
    renderer::RenderCommand,
    widget::Element,
};

#[derive(Default)]
pub struct Context {
    event_buffer: Vec<InputEvent>,
    input_state: InputState,
    dpi: f32,
}
impl Context {
    pub fn dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }

    pub fn layout<T: 'static>(&mut self, widget: Element<T>) -> LayoutResult<T> {
        let mut node = crate::layout::layout(widget, self.dpi);

        let mut responses = vec![];
        std::mem::take(&mut self.event_buffer)
            .into_iter()
            .for_each(|event| {
                node.event(event, &self.input_state, &mut responses);
            });

        let render_commands = node.render();

        LayoutResult {
            render_commands,
            responses,
        }
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.input_state.event(event);
        self.event_buffer.push(event);
    }
}

pub struct LayoutResult<T> {
    pub render_commands: Vec<RenderCommand>,
    pub responses: Vec<T>,
}
