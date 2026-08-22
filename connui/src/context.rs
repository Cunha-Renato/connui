use crate::{
    event::{Event, InputEvent, InputState},
    renderer::Renderer,
    state::StateContext,
    tree::{Element, layout::LayoutElementTree, widget::Widget},
};

pub struct Context<T, R: Renderer> {
    state_context: StateContext<R>,

    layout_tree: LayoutElementTree,
    tree: Option<Element<T, R>>,

    input_buffer: Vec<InputEvent>,

    curr_input_state: InputState,
    prev_input_state: InputState,
}
impl<T, R: Renderer> Context<T, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),

            layout_tree: LayoutElementTree::default(),
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
}
impl<T: 'static, R: Renderer + 'static> Context<T, R> {
    pub fn layout(&mut self, widget: Widget<T, R>) {
        let tree = match &mut self.tree {
            Some(tree) => {
                tree.reconcile(&mut self.layout_tree, widget);
                tree
            }
            None => {
                self.tree = Some(widget.mount());
                self.tree.as_mut().unwrap()
            }
        };

        self.layout_tree.layout(tree, 5);

        tree.render(&self.layout_tree, self.state_context.renderer_mut());

        self.prev_input_state = self.curr_input_state.clone();
    }

    #[inline]
    pub fn event<F>(&mut self, event: InputEvent, mut handler: F)
    where
        F: FnMut(T),
    {
        self.curr_input_state.event(event);
        self.input_buffer.push(event);

        let mut responses = vec![];
        // if let Some(tree) = &mut self.tree {
        //     Event::generate_recursive(
        //         event,
        //         &self.prev_input_state,
        //         &self.curr_input_state,
        //         tree,
        //         &self.layout_tree,
        //         &mut responses,
        //     );
        // }

        for response in responses {
            handler(response);
        }
    }
}
