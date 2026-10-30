use crate::{
    event::InputEvent,
    renderer::Renderer,
    state::StateContext,
    tree::{
        Element, LayoutElement, LayoutElementRef, LayoutElementWeak, visual::VisualElement,
        widget::Widget,
    },
};

pub struct Context<T, R: Renderer> {
    state_context: StateContext<R>,
    element_tree: Option<Element<T, R>>,
    layout_tree: LayoutElementRef,
}
impl<T, R: Renderer> Context<T, R> {
    pub fn new(renderer: R) -> Self {
        Self {
            state_context: StateContext::new(renderer),
            element_tree: None,
            layout_tree: LayoutElement::new_ref(&Default::default()),
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
        let element_tree = match &mut self.element_tree {
            Some(element_tree) => {
                element_tree.reconcile(widget);

                element_tree
            }
            None => {
                let mut element_tree = widget.mount();
                self.layout_tree = LayoutElement::new_ref(element_tree.element.style());
                element_tree.layout_element = LayoutElementRef::downgrade(&self.layout_tree);
                LayoutElement::populate_element(&mut element_tree, &LayoutElementWeak::new());

                self.element_tree = Some(element_tree);
                self.element_tree.as_mut().unwrap()
            }
        };

        LayoutElement::layout(element_tree, &self.layout_tree);

        let mut visual_tree = VisualElement::new_root(element_tree);
        visual_tree.render(self.state_context.renderer_mut());

        self.state_context.input_mut().send_input(&mut visual_tree)
    }

    #[inline]
    pub fn event(&mut self, event: InputEvent) {
        self.state_context.input_mut().process_incoming(event);
    }
}
