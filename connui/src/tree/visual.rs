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

    pub(crate) fn process_input(
        &mut self,
        layout_tree: &LayoutElementTree,
        input_context: &mut InputContext,
    ) -> Vec<T> {
        // First the elements that capture / focus inputs.
        // Due to it this is a possible double pass for every event.
        let events = input_context.events();
        let mut responses = vec![];

        if events.is_empty() {
            return responses;
        }

        // Most of the times this has len of 1.
        for event in events {
            // Signals that last frame input capture was requested.
            // We need to find who requested it and process it first.
            if input_context.capture.intersects(event.kind()) {
                self.process_input_capture(
                    event,
                    layout_tree,
                    &mut input_context.capture,
                    &mut responses,
                );
            } else {
                self.process_input_all(
                    event,
                    layout_tree,
                    &mut input_context.capture,
                    &mut responses,
                );
            }
        }

        responses
    }

    /// Returns **true** if the event was consumed.
    fn process_input_individual(
        &mut self,
        event: Event,
        layout_tree: &LayoutElementTree,
        input_capture: &mut EventKind,
        responses: &mut Vec<T>,
    ) -> bool {
        let layout_element = &layout_tree[self.layout_key];

        let response = self.element.input_event(event, layout_element);
        let consumed = response.consumed();

        if let Some(msg) = response.take() {
            responses.push(msg);
        }

        // Only (re)register capture if this element actually did something with
        // the event, so capture state doesn't get silently re-armed every frame
        // regardless of whether anything happened.
        if consumed {
            input_capture.insert(self.element.input_capture());
        }

        consumed
    }

    fn process_input_all(
        &mut self,
        event: Event,
        layout_tree: &LayoutElementTree,
        input_capture: &mut EventKind,
        responses: &mut Vec<T>,
    ) -> bool {
        let kind = event.kind();
        let mut consumed = false;

        // Children first.
        for child in self.children.iter_mut().rev() {
            if consumed {
                child.element.input_consumed(kind);
            } else {
                consumed |= child.process_input_all(event, layout_tree, input_capture, responses);
            }
        }

        if consumed {
            self.element.input_consumed(kind);
        } else {
            consumed |= self.process_input_individual(event, layout_tree, input_capture, responses);
        }

        consumed
    }

    /// Returns **true** if the event was consumed by the captured element.
    fn process_input_capture(
        &mut self,
        event: Event,
        layout_tree: &LayoutElementTree,
        input_capture: &mut EventKind,
        responses: &mut Vec<T>,
    ) -> bool {
        self.find_element(&mut |el| input_capture.intersects(el.element.input_capture()))
            .is_some_and(|element| {
                // Should this be Self::process_input_all?
                let consumed =
                    element.process_input_individual(event, layout_tree, input_capture, responses);
                if consumed {
                    *input_capture = input_capture.intersection(element.element.input_capture());
                }

                consumed
            })
    }

    /// Finds the first [`VisualElement`] in the tree that matches the predicate.
    ///
    /// Searches children in reverse order ie: visual order.
    fn find_element<F: FnMut(&mut Self) -> bool>(
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

    /// Overlay children are relative to root element.
    pub(crate) fn new_root(root: &'a mut Element<T, R>) -> Self {
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
