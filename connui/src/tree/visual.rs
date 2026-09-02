use crate::{renderer::Renderer, types::*};

use super::*;

/// Struct that reorders [`Element`]s according to [`Position`].
/// Only used in rendering and input processing.
///
/// [`Position::Dynamic`] comes first, then [`Position::Pinned`], lastly [`PinnedFlags::OVERLAY`]
/// are placed at the end of the root [`Element`].
pub struct VisualElement<'a, T, R: Renderer> {
    pub(crate) children: Vec<Self>,
    pub(crate) element: &'a mut dyn ElementSpecs<T, R>,
    pub(crate) layout_key: LayoutElementKey,
}
impl<'a, T: 'static, R: Renderer + 'static> VisualElement<'a, T, R> {
    pub fn render(&self, parent_context: LayoutContextRef, renderer: &mut R) {
        let layout_tree = parent_context.layout_tree;

        let render_element = RenderElement::new(&layout_tree[self.layout_key], renderer);
        let context = LayoutContextRef::new(self.layout_key, layout_tree);

        self.element
            .render(context, &self.children, render_element, renderer);
    }

    /// Overlay children are relative to root element.
    pub(crate) fn new_root(root: &'a mut Element<T, R>) -> Self {
        let mut overlay = vec![];
        let mut root = Self::new_children(root, &mut overlay);
        root.children.extend(overlay);

        root
    }

    /// Finds the first [`VisualElement`] in the tree that matches the predicate.
    ///
    /// Searches children in reverse order ie: visual order.
    pub(crate) fn find_element<F: FnMut(&mut Self) -> bool>(
        &mut self,
        predicate: &mut F,
    ) -> Option<&mut Self> {
        // Note: Fuck the borrow checker.
        // TODO: Test the version with find_map once the new compiler arrives.

        let self_ptr: *mut Self = self;

        for child in unsafe { (*self_ptr).children.iter_mut().rev() } {
            if let Some(found) = child.find_element(predicate) {
                return Some(found);
            }
        }

        predicate(self).then_some(self)
    }

    #[inline]
    pub(crate) fn mouse_event(
        &mut self,
        event: MouseEvent,
        layout_tree: &mut LayoutElementTree,
    ) -> Response<T> {
        let context = LayoutContext::new(self.layout_key, layout_tree);
        self.element.mouse_event(event, context)
    }

    #[inline]
    pub(crate) fn window_event(
        &mut self,
        event: WindowEvent,
        layout_tree: &mut LayoutElementTree,
    ) -> Option<T> {
        let context = LayoutContext::new(self.layout_key, layout_tree);
        self.element.window_event(event, context)
    }

    fn new_children(element: &'a mut Element<T, R>, overlay: &mut Vec<Self>) -> Self {
        let (pinned, mut remaining): (Vec<_>, Vec<_>) = element
            .children
            .iter_mut()
            .partition(|child| child.element.style().position.is_pinned());

        for pinned_child in pinned {
            let Position::Pinned { flags, .. } = pinned_child.element.style().position else {
                unreachable!("partition guarantees Position::Pinned here")
            };

            if flags.contains(PinnedFlags::OVERLAY) {
                // Computes the overlay children first so that any overlay inside it gets extracted.
                let overlay_child = Self::new_children(pinned_child, overlay);
                overlay.push(overlay_child);
            } else {
                // Pushed to the back of element's children.
                remaining.push(pinned_child)
            }
        }

        Self {
            children: remaining
                .into_iter()
                .map(|r| Self::new_children(r, overlay))
                .collect(),
            element: element.element.as_mut(),
            layout_key: element.layout.unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct RenderElement {
    pub rect: PRect,
    pub scissor: Rect<u32, u32>,
}
impl RenderElement {
    /// QOL function to check if the scissor is visible.
    #[inline]
    pub const fn can_render_children(&self) -> bool {
        self.scissor.x() < self.scissor.width() && self.scissor.y() < self.scissor.width()
    }

    pub(crate) fn new<R: Renderer>(layout_element: &LayoutElement, renderer: &mut R) -> Self {
        let scale_factor = renderer.scale_factor();

        let rect = layout_element.rect.map(|r| r.to_physical(scale_factor));
        let clip_inner = layout_element
            .clip
            .map(|c| c.to_physical(scale_factor).inner());

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
