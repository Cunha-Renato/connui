use crate::{
    event::{InputEvent, InputState},
    renderer::Renderer,
    state::StateContext,
    types::Node,
    widget::Element,
};

pub struct Context<R: Renderer> {
    state_context: StateContext<R>,
    curr_input_state: InputState,
    prev_input_state: InputState,
    scale_factor: f32,
}
impl<R: Renderer + 'static> Context<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),
            curr_input_state: Default::default(),
            prev_input_state: Default::default(),
            scale_factor: Default::default(),
        }
    }

    pub fn scale_factor(mut self, scale_factor: f32) -> Self {
        self.scale_factor = scale_factor;
        self
    }

    #[inline]
    pub fn renderer_ref(&self) -> &R {
        self.state_context.renderer()
    }

    #[inline]
    pub fn renderer_mut(&mut self) -> &mut R {
        self.state_context.renderer_mut()
    }

    pub fn layout<T: 'static>(&mut self, widget: Element<T, R>) -> Vec<T> {
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

        node.render(&mut self.state_context);

        responses
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.curr_input_state.event(event);
    }
}
