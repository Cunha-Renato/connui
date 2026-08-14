use crate::{
    event::{InputEvent, InputState},
    layout::LayoutElement,
    renderer::Renderer,
    state::StateContext,
    types::Rect,
    widget::{Element, diff::DiffElement},
};

pub struct Context<R: Renderer> {
    state_context: StateContext<R>,

    layout_tree: Option<LayoutElement>,
    diff_tree: Option<DiffElement>,

    curr_input_state: InputState,
    prev_input_state: InputState,
}
impl<R: Renderer> Context<R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),
            curr_input_state: Default::default(),
            prev_input_state: Default::default(),
            diff_tree: None,
            layout_tree: None,
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

    pub fn layout<T: 'static>(&mut self, mut widget: Element<T, R>) -> Vec<T> {
        widget.init(&mut self.state_context);

        if self
            .diff_tree
            .as_ref()
            .map(|prev| !widget.diff(prev))
            .is_none_or(|val| val)
        {
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

        // if node.update(&mut self.state_context) {
        //     node.layout();
        //     node.update(&mut self.state_context);
        // }
        // println!("{node:#?}");

        let mut responses = vec![];
        // TODO: Mouse pos must be in logical pixels.
        // node.event(
        //     &self.prev_input_state,
        //     &self.curr_input_state,
        //     &mut responses,
        // );
        self.prev_input_state = self.curr_input_state.clone();
        self.curr_input_state.next_frame();

        widget.render(
            self.layout_tree.as_ref().unwrap(),
            self.state_context.renderer_mut(),
        );

        self.diff_tree = Some(widget.into());

        responses
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.curr_input_state.event(event);
    }
}
