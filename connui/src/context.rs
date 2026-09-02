use crate::{
    event::InputEvent,
    renderer::Renderer,
    state::StateContext,
    tree::{
        Element, LayoutContextRef, layout::LayoutElementTree, visual::VisualElement, widget::Widget,
    },
};

pub struct Context<T, R: Renderer> {
    state_context: StateContext<R>,

    layout_tree: LayoutElementTree,
    tree: Option<Element<T, R>>,
}
impl<T, R: Renderer> Context<T, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),

            layout_tree: LayoutElementTree::default(),
            tree: None,
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
    pub fn layout(&mut self, widget: Widget<T, R>) -> Vec<T> {
        let element_tree = match &mut self.tree {
            Some(tree) => {
                tree.reconcile(&mut self.layout_tree, widget);
                tree
            }
            None => {
                self.layout_tree.dirty();
                self.tree = Some(widget.mount());
                self.tree.as_mut().unwrap()
            }
        };

        self.layout_tree.layout_root(element_tree);

        let mut visual_tree = VisualElement::new_root(element_tree);
        visual_tree.render(
            LayoutContextRef::new(visual_tree.layout_key, &self.layout_tree),
            self.state_context.renderer_mut(),
        );
        self.state_context
            .input_mut()
            .send_input(&mut visual_tree, &mut self.layout_tree)
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.state_context.input_mut().process_incoming(event);
    }
}
