pub mod layout;
pub mod widget;

use std::any::Any;

use crate::{
    event::{Event, InputEvent, InputState, MouseEvent, MouseInputEvent},
    renderer::Renderer,
    types::Response,
};

pub use layout::*;
pub use widget::*;

#[repr(transparent)]
struct Differ<'a>(&'a dyn Any);
impl PartialEq<Key> for Differ<'_> {
    #[inline]
    fn eq(&self, key: &Key) -> bool {
        key == self.0.type_id()
    }
}

#[repr(transparent)]
pub struct Updater<'a>(&'a mut dyn Any);
impl<'a> Updater<'a> {
    #[inline]
    pub fn update<W, E: 'static, F>(self, widget: W, f: F)
    where
        F: FnOnce(W, &mut E),
    {
        if let Some(element) = self.0.downcast_mut::<E>() {
            f(widget, element)
        }
    }
}

pub trait ElementSpecs<T, R: Renderer>: Any {
    fn style(&self) -> &Style;

    #[allow(unused_variables)]
    fn layout(&mut self, layout_key: LayoutElementKey, layout_tree: &mut LayoutElementTree) {}

    fn event(&mut self, event: Event, layout_element: &LayoutElement) -> Response<T>;

    fn render(
        &self,
        children: &[Element<T, R>],
        layout_element: &LayoutElement,
        layout_tree: &LayoutElementTree,
        renderer: &mut R,
    );
}

pub struct Element<T, R: Renderer> {
    pub(crate) children: Vec<Self>,
    element: Box<dyn ElementSpecs<T, R>>,
    layout: Option<LayoutElementKey>,
}
impl<T: 'static, R: Renderer + 'static> Element<T, R> {
    pub fn render(&self, layout_tree: &LayoutElementTree, renderer: &mut R) {
        todo!();
        if let Some(layout_key) = self.layout {
            self.element.render(
                &self.children,
                &layout_tree[layout_key],
                layout_tree,
                renderer,
            );
        }
    }

    #[inline]
    pub(crate) fn on_event(
        &mut self,
        layout_tree: &LayoutElementTree,
        event: InputEvent,
        prev_state: &InputState,
        curr_state: &InputState,
    ) -> Response<T> {
        let layout_element = if let Some(layout_key) = self.layout {
            &layout_tree[layout_key]
        } else {
            eprint!("Element::on_event without layout!");

            return Default::default();
        };

        match event {
            InputEvent::Mouse(mouse_input_event) => {
                // Children First.
                let prev_inside = layout_element.clip.is_inside(
                    prev_state.mouse_position().x.as_float(),
                    prev_state.mouse_position().y.as_float(),
                );
                let curr_inside = layout_element.clip.is_inside(
                    curr_state.mouse_position().x.as_float(),
                    curr_state.mouse_position().y.as_float(),
                );

                match mouse_input_event {
                    MouseInputEvent::Button { button, pressed } => {
                        let mouse_event = if pressed {
                            MouseEvent::Press(button)
                        } else {
                            MouseEvent::Release(button)
                        };

                        self.element.event(
                            Event::Mouse {
                                event: mouse_event,
                                position: curr_state.mouse_position(),
                            },
                            layout_element,
                        )
                    }
                    MouseInputEvent::Move(point) => {
                        let event = if prev_inside && !curr_inside {
                            MouseEvent::Leave
                        } else if !prev_inside && curr_inside {
                            MouseEvent::Enter
                        } else {
                            MouseEvent::Move
                        };

                        self.element.event(
                            Event::Mouse {
                                event,
                                position: point,
                            },
                            layout_element,
                        )
                    }
                    MouseInputEvent::Scroll(delta) => self.element.event(
                        Event::Mouse {
                            event: MouseEvent::Scroll(delta),
                            position: curr_state.mouse_position(),
                        },
                        layout_element,
                    ),
                }
            }
        }
    }

    pub(crate) fn layout(&mut self, layout_tree: &mut LayoutElementTree) {
        self.element.layout(self.layout.unwrap(), layout_tree);

        for child in &mut self.children {
            child.layout(layout_tree);
        }
    }

    pub(crate) fn reconcile(
        &mut self,
        layout_tree: &mut LayoutElementTree,
        mut widget: Widget<T, R>,
    ) {
        if self.differ() == widget.key() {
            let children = widget.children();

            widget.update(self.updater());
            self.validate_style(layout_tree);
            self.reconcile_children(layout_tree, children);
        } else {
            *self = widget.mount();
            layout_tree.dirty();
        }
    }

    fn reconcile_children(
        &mut self,
        layout_tree: &mut LayoutElementTree,
        new_children: Vec<Widget<T, R>>,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        } else {
            layout_tree.dirty();
        }

        let mut new_children = new_children.into_iter();

        for (old, new) in self.children.iter_mut().zip(new_children.by_ref()) {
            old.reconcile(layout_tree, new);
        }

        self.children.extend(new_children.map(Widget::mount));
    }

    /// Marks tree as dirty if [`Style`] is different from [`LayoutElement`].
    fn validate_style(&mut self, layout_tree: &mut LayoutElementTree) {
        // Should always be Some.
        if let Some(layout_key) = self.layout {
            let inner_style = self.element.style().into();

            if layout_tree[layout_key].style == inner_style {
                layout_tree[layout_key].style = inner_style;
                layout_tree.dirty();
            }
        }
    }
}
impl<T, R: Renderer> Element<T, R> {
    #[inline]
    pub fn new<E, I>(children: I, element: E) -> Self
    where
        E: ElementSpecs<T, R> + 'static,
        I: IntoIterator<Item = Widget<T, R>>,
    {
        Self {
            children: children.into_iter().map(Widget::mount).collect(),
            element: Box::new(element),
            layout: None,
        }
    }

    #[inline]
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    #[inline]
    fn differ<'a>(&'a self) -> Differ<'a> {
        Differ(self.element.as_ref())
    }

    #[inline]
    fn updater<'a>(&'a mut self) -> Updater<'a> {
        Updater(self.element.as_mut())
    }
}
