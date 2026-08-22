use crate::{renderer::Renderer, types::*};

use super::*;

/// Struct that reorders [`Element`]s according to [`Position`].
/// Only used in rendering and input processing.
///
/// [`Position::Dynamic`] comes first, then [`Position::Pinned`], lastly [`PinnedFlags::OVERLAY`]
/// are placed at the end of the root [`Element`].
pub struct VisualElement<'a, T, R: Renderer> {
    children: Vec<Self>,
    element: &'a mut dyn ElementSpecs<T, R>,
    layout_key: LayoutElementKey,
}
impl<'a, T: 'static, R: Renderer + 'static> VisualElement<'a, T, R> {
    pub fn render(&self, layout_tree: &LayoutElementTree, renderer: &mut R) {
        let render_element = RenderElement::new(&layout_tree[self.layout_key], renderer);

        self.element
            .render(&self.children, render_element, layout_tree, renderer);
    }

    /// Overlay children are relative to root element.
    pub(crate) fn root(root: &'a mut Element<T, R>) -> Self {
        let mut overlay = vec![];
        let mut root = Self::new_children(root, &mut overlay);
        root.children.extend(overlay);

        root
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

    fn new<R: Renderer>(layout_element: &LayoutElement, renderer: &mut R) -> Self {
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
