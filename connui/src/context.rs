use crate::{
    event::{Event, InputEvent, InputState},
    layout::LayoutElement,
    renderer::Renderer,
    state::StateContext,
    types::Rect,
    widget::Element,
};

pub struct Context<T, R: Renderer> {
    state_context: StateContext<R>,

    layout_tree: Option<LayoutElement>,
    tree: Option<Element<T, R>>,

    input_buffer: Vec<InputEvent>,

    curr_input_state: InputState,
    prev_input_state: InputState,
}
impl<T, R: Renderer> Context<T, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),

            layout_tree: None,
            tree: None,

            input_buffer: vec![],

            curr_input_state: Default::default(),
            prev_input_state: Default::default(),
        }
    }

    pub fn scale_factor(mut self, scale_factor: f32) -> Self {
        self.renderer_mut().set_scale_factor(scale_factor);
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

    pub fn layout(&mut self, mut widget: Element<T, R>) -> Vec<T> {
        let mut relayout = false;
        widget.init(self.tree.as_ref(), &mut relayout, &mut self.state_context);

        if relayout {
            let mut layout_tree = widget.layout();

            let mut max_tries = 5;
            while layout_tree.is_dirty() && max_tries > 0 {
                layout_tree = widget.layout();
                max_tries -= 1;
            }

            layout_tree.clip(&Rect::new(
                0.0.into(),
                0.0.into(),
                f32::MAX.into(),
                f32::MAX.into(),
            ));

            self.layout_tree = Some(layout_tree);
        }

        let mut responses = vec![];
        // TODO: Mouse pos must be in logical pixels.

        for input in std::mem::take(&mut self.input_buffer) {
            Event::generate_recursive(
                input,
                &self.prev_input_state,
                &self.curr_input_state,
                &mut widget,
                self.layout_tree.as_ref().unwrap(),
                &mut responses,
            );
        }

        self.prev_input_state = self.curr_input_state.clone();

        widget.render(
            self.layout_tree.as_ref().unwrap(),
            self.state_context.renderer_mut(),
        );

        self.tree = Some(widget);

        responses
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.curr_input_state.event(event);
        self.input_buffer.push(event);
    }
}
