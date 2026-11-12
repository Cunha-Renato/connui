use crate::{renderer::Renderer, types::*};

use super::*;

/// Struct that reorders [`Element`]s according to [`Positioning`].
/// Only used in rendering and input processing.
///
/// [`Positioning::Dynamic`] comes first, then [`Positioning::Pinned`], lastly [`PinnedFlags::OVERLAY`]
/// are placed at the end of the root [`Element`].
pub struct VisualElement<'a, T, R: Renderer> {
    pub(crate) children: Vec<Self>,
    pub(crate) element: &'a mut dyn ElementSpecs<T, R>,
    pub(crate) parent: Option<std::ptr::NonNull<Self>>,
    pub(crate) layout_element: LayoutElementRef,
}
impl<'a, T: 'static, R: Renderer + 'static> VisualElement<'a, T, R> {
    pub fn render(&self, renderer: &mut R) {
        let render_element = RenderElement::new(self.layout_element.clone(), renderer);

        self.element.render(
            &self.layout_element,
            &self.children,
            render_element,
            renderer,
        );
    }

    /// Overlay children are relative to root element.
    pub(crate) fn new_root(root: &'a mut Element<T, R>) -> Box<Self> {
        let mut overlay = vec![];
        let mut root_visual = Box::new(Self::new(
            root.element.as_mut(),
            root.layout_element.upgrade().unwrap(),
        ));
        root_visual.new_children(root.children.iter_mut(), &mut overlay);
        root_visual.children.extend(overlay);
        root_visual.link_parents();

        root_visual
    }

    /// Finds the first [`VisualElement`] in the tree that matches the predicate.
    ///
    /// Searches children in reverse order ie: visual order.
    pub(crate) fn find_element<F: FnMut(&mut Self) -> bool>(
        &mut self,
        predicate: &mut F,
    ) -> Option<&mut Self> {
        let self_ptr: *mut Self = self;
        unsafe {
            (*self_ptr)
                .children
                .iter_mut()
                .rev()
                .find_map(|child| child.find_element(predicate))
                .or_else(|| predicate(self).then_some(self))
        }
    }

    #[inline]
    pub(crate) fn mouse_event(&mut self, event: MouseEvent) -> Response<T> {
        self.element.mouse_event(event, &self.layout_element)
    }

    #[inline]
    pub(crate) fn window_event(&mut self, event: WindowEvent) -> Option<T> {
        self.element.window_event(event, &self.layout_element)
    }

    fn new(element: &'a mut dyn ElementSpecs<T, R>, layout_element: LayoutElementRef) -> Self {
        Self {
            children: vec![],
            element,
            parent: None,
            layout_element,
        }
    }

    fn new_children(
        &mut self,
        children: impl IntoIterator<Item = &'a mut Element<T, R>>,
        overlay: &mut Vec<Self>,
    ) {
        let remaining = children
            .into_iter()
            .filter_map(|child| {
                let layout_element = child.layout_element.upgrade().unwrap();

                { layout_element.borrow().is_visible() }.then_some((child, layout_element))
            })
            .filter_map(|(child, layout_element)| {
                let position = child.element.style().position;

                let mut visual_child = Self::new(child.element.as_mut(), layout_element);
                visual_child.new_children(&mut child.children, overlay);

                match position {
                    Positioning::Pinned { flags, .. } if flags.contains(PinnedFlags::OVERLAY) => {
                        overlay.push(visual_child);

                        None
                    }
                    _ => Some(visual_child),
                }
            })
            .collect::<Vec<_>>();

        self.children = remaining;
    }

    fn link_parents(&mut self) {
        let parent = std::ptr::NonNull::from_mut(self);
        for child in &mut self.children {
            child.parent = Some(parent);
            child.link_parents();
        }
    }
}

#[derive(Debug)]
pub struct RenderElement {
    pub rect: PRect,
    pub scissor: Rect<u32, u32>,
}
impl RenderElement {
    pub(crate) fn new<R: Renderer>(layout_element: LayoutElementRef, renderer: &mut R) -> Self {
        let borrow = layout_element.borrow();
        let scale_factor = renderer.scale_factor();

        let rect = borrow.rect.map(|r| r.to_physical(scale_factor));
        let clip_inner = borrow.clip.map(|c| c.to_physical(scale_factor).inner());

        let x0 = clip_inner.x();
        let y0 = clip_inner.y();
        let x1 = clip_inner.x() + clip_inner.width();
        let y1 = clip_inner.y() + clip_inner.height();

        let scissor_x = x0.floor().max(0.0) as u32;
        let scissor_y = y0.floor().max(0.0) as u32;
        let scissor_w = (x1 - x0).ceil().max(0.0) as u32;
        let scissor_h = (y1 - y0).ceil().max(0.0) as u32;

        let scissor = Rect::new(scissor_x, scissor_y, scissor_w, scissor_h);

        Self { rect, scissor }
    }
}
