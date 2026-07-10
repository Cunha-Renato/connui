use crate::{
    event::{InputEvent, InputState},
    renderer::RenderCommand,
    state::StateContext,
    types::Node,
    widget::Element,
};

#[derive(Default)]
pub struct Context {
    state_context: StateContext,
    curr_input_state: InputState,
    prev_input_state: InputState,
    scale_factor: f32,
}
impl Context {
    pub fn scale_factor(mut self, scale_factor: f32) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    pub fn layout<T: 'static>(&mut self, widget: Element<T>) -> LayoutResult<T> {
        let mut node = Node::from_element(
            widget,
            &mut Some(&mut self.state_context),
            &Default::default(),
            Default::default(),
        );
        crate::layout::layout(&mut node, self.scale_factor);

        let mut responses = vec![];
        node.event(
            &self.prev_input_state,
            &self.curr_input_state,
            &mut responses,
        );
        self.prev_input_state = self.curr_input_state.clone();
        self.curr_input_state.next_frame();

        let render_commands = node.render(&mut self.state_context);

        LayoutResult {
            render_commands,
            responses,
        }
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.curr_input_state.event(event);
    }
}

pub struct LayoutResult<T> {
    pub render_commands: Vec<RenderCommand>,
    pub responses: Vec<T>,
}
