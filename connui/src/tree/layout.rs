use std::{
    cell::{Ref, RefCell, RefMut},
    rc::{Rc, Weak},
};

use crate::{
    has_layout, has_margin, has_padding, has_positioning, has_sizing, renderer::Renderer,
    tree::RenderElement, types::*,
};

use super::Element;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub size: Size<Sizing>,
    pub padding: LSides<u16>,
    pub margin: LSides<u16>,
    pub position: Positioning,
    pub hor_align: HorAlign,
    pub ver_align: VerAlign,
    pub layout: Layout,
}
impl Style {
    pub const fn new() -> Self {
        Self {
            size: Size::new(Sizing::fit_default(), Sizing::fit_default()),
            padding: Sides::all(LPixel::new(0)),
            margin: Sides::all(LPixel::new(0)),
            position: Positioning::Dynamic,
            hor_align: HorAlign::Left,
            ver_align: VerAlign::Top,
            layout: Layout {
                axis: LayoutAxis::Vertical,
            },
        }
    }
}
impl Default for Style {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
has_sizing!(Style with {size});
has_padding!(Style with {padding});
has_margin!(Style with {margin});
has_positioning!(Style with {position});
has_layout!(Style with {layout});

#[derive(PartialEq)]
pub struct InnerStyle {
    pub size: Size<Sizing>,
    pub padding: LSides<f32>,
    pub margin: LSides<f32>,
    pub position: Positioning,
    pub hor_align: HorAlign,
    pub ver_align: VerAlign,
    pub layout: Layout,
}
impl InnerStyle {
    fn to_style(&self) -> Style {
        Style::from(self)
    }
}
impl From<Style> for InnerStyle {
    #[inline]
    fn from(value: Style) -> Self {
        Self::from(&value)
    }
}
impl From<&Style> for InnerStyle {
    #[inline]
    fn from(value: &Style) -> Self {
        Self {
            size: value.size,
            padding: value.padding.map(|p| p.as_float()),
            margin: value.margin.map(|m| m.as_float()),
            position: value.position,
            hor_align: value.hor_align,
            ver_align: value.ver_align,
            layout: value.layout,
        }
    }
}
impl From<InnerStyle> for Style {
    fn from(value: InnerStyle) -> Self {
        Self::from(&value)
    }
}
impl From<&InnerStyle> for Style {
    fn from(value: &InnerStyle) -> Self {
        Self {
            size: value.size,
            padding: value.padding.map(|p| p.as_unsigned()),
            margin: value.margin.map(|m| m.as_unsigned()),
            position: value.position,
            hor_align: value.hor_align,
            ver_align: value.ver_align,
            layout: value.layout,
        }
    }
}

pub struct LayoutElementData {
    pub bounds: Bounds,
    pub rect: LRect<f32, f32>,
    pub clip: LRect<f32, f32>,
    dirty: bool,
}
impl LayoutElementData {
    pub(crate) fn clear(&mut self) {
        self.dirty = false;
        self.bounds = Bounds::default();
        self.rect = Rect::default();
        self.clip = Rect::default();
    }
}
impl Default for LayoutElementData {
    fn default() -> Self {
        Self {
            bounds: Bounds::default(),
            rect: Rect::default(),
            clip: Rect::default(),
            dirty: true,
        }
    }
}

pub struct LayoutElement {
    pub(crate) style: RefCell<InnerStyle>,
    pub(crate) data: RefCell<LayoutElementData>,
    pub(crate) link: RefCell<LayoutElementLink>,
}
impl LayoutElement {
    #[inline]
    pub fn new(style: impl Into<InnerStyle>) -> Self {
        Self {
            style: RefCell::new(style.into()),
            data: RefCell::new(LayoutElementData::default()),
            link: RefCell::new(LayoutElementLink::default()),
        }
    }

    #[inline]
    pub fn new_ref(style: impl Into<InnerStyle>) -> LayoutElementRef {
        LayoutElementRef::new(Self::new(style))
    }

    #[inline]
    pub fn render_element<R: Renderer>(self: &LayoutElementRef, renderer: &mut R) -> RenderElement {
        RenderElement::new(self.clone(), renderer)
    }

    #[inline]
    pub fn data(&self) -> Ref<'_, LayoutElementData> {
        self.data
            .try_borrow()
            .expect("LayoutElement::data while LayoutElement::data_mut was active!")
    }

    #[inline]
    pub fn data_mut(&self) -> RefMut<'_, LayoutElementData> {
        self.data
            .try_borrow_mut()
            .expect("LayoutElement::data_mut while borrow was active!")
    }

    #[inline]
    pub fn style(&self, f: impl Fn(Style) -> Style) {
        let old_style = self
            .style
            .replace_with(|old| f(InnerStyle::to_style(old)).into());

        if &old_style != &*self.style.borrow() {
            self.mark_dirty();
        }
    }

    pub fn children<'a, F, R>(&'a self, mut f: F) -> R
    where
        F: FnMut(&[LayoutElementRef]) -> R,
    {
        let link_borrow = self.link.borrow();
        f(&link_borrow.children)
    }

    pub fn push_child(self: &LayoutElementRef, child: LayoutElementRef) {
        child.link.borrow_mut().parent = Rc::downgrade(self);
        self.link.borrow_mut().children.push(child);
    }

    pub fn extend_children<I: IntoIterator<Item = LayoutElementRef>>(
        self: &LayoutElementRef,
        children: I,
    ) {
        let iter = children.into_iter().map(|child| {
            child.link.borrow_mut().parent = Rc::downgrade(self);
            child
        });
        self.link.borrow_mut().children.extend(iter);
    }

    pub fn set_children(self: &LayoutElementRef, children: Vec<LayoutElementRef>) {
        children
            .iter()
            .for_each(|child| child.link.borrow_mut().parent = Rc::downgrade(self));
        self.link.borrow_mut().children = children;
    }

    pub fn swap_children(self: &LayoutElementRef, dst: &LayoutElementRef) {
        let mut this_link = self.link.borrow_mut();
        let mut dst_link = dst.link.borrow_mut();

        for this_child in &this_link.children {
            this_child.link.borrow_mut().parent = Rc::downgrade(dst);
        }

        for dst_child in &dst_link.children {
            dst_child.link.borrow_mut().parent = Rc::downgrade(self);
        }

        std::mem::swap(&mut this_link.children, &mut dst_link.children);
    }

    pub fn mark_dirty(&self) {
        self.data_mut().dirty = true;

        let style = self.style.borrow();

        // Parent.
        if let Some(parent) = self.link.borrow().parent.upgrade()
            && (!style.size.width.is_absolute() || !style.size.height.is_absolute())
            && !style.position.is_pinned()
        {
            if !parent.data().dirty {
                parent.mark_dirty();
            }
        }
    }

    pub(crate) fn layout<T: 'static, R: Renderer + 'static>(
        root_element: &mut Element<T, R>,
        root_layout: &LayoutElementRef,
    ) {
        fn pass<T: 'static, R: Renderer + 'static>(
            root_element: &mut Element<T, R>,
            root_layout: &LayoutElementRef,
            depth: usize,
            first_pass: bool,
        ) {
            if root_layout.data().dirty {
                if first_pass {
                    println!("RUNNING FIRST PASS LAYOUT AT DEPTH: {depth}");
                    LayoutElement::repopulate_element(root_element);
                } else {
                    println!("RUNNING SECOND PASS LAYOUT AT DEPTH: {depth}");
                }

                let parent_layout_weak = root_layout.link.borrow().parent.clone();

                let parent_layout = parent_layout_weak
                    .upgrade()
                    .unwrap_or_else(|| root_layout.clone());
                let parent_bounds = parent_layout.data().bounds.clone();

                if first_pass {
                    root_element.pre_layout();
                }
                crate::layout::measure(&root_layout, &parent_bounds);
                crate::layout::resolve_children_size(&parent_layout);
                crate::layout::resolve_children_position(&parent_layout);
                if first_pass {
                    root_element.post_layout();
                }
            } else {
                for child in &mut root_element.children {
                    pass(
                        child,
                        &child.layout_element.upgrade().unwrap(),
                        depth + 1,
                        first_pass,
                    );
                }
            }
        }

        pass(root_element, root_layout, 0, true);
        pass(root_element, root_layout, 0, false);
        root_layout.resolve_clip();
    }

    pub(crate) fn resolve_clip(self: &LayoutElementRef) {
        // First against the root itself.
        let max_rect = Rect::new(0.0.into(), 0.0.into(), f32::MAX.into(), f32::MAX.into());
        self.calculate_clip(&max_rect, &max_rect);

        let root_clip = self.data().clip.clone();
        self.resolve_clip_inner(&root_clip);
    }

    /// Rebuilds the layout node for `element` (and its subtree), reusing nodes
    /// that already exist. Returns the strong ref: if `parent_layout` is dead
    /// (i.e. this is a root), the caller must keep the returned `Rc` alive.
    pub(crate) fn populate_element<T: 'static, R: Renderer + 'static>(
        element: &mut Element<T, R>,
        parent_layout: &LayoutElementWeak,
    ) -> LayoutElementRef {
        let layout_element = element
            .layout_element
            .upgrade()
            .unwrap_or_else(|| Self::new_ref(element.element.style()));

        // Attach to the parent once (no duplicates, no reordering).
        if let Some(parent) = parent_layout.upgrade() {
            let attached = parent
                .link
                .borrow()
                .children
                .iter()
                .any(|c| Rc::ptr_eq(c, &layout_element));

            if !attached {
                parent.push_child(layout_element.clone());
            }
        }

        Self::rebuild_children(element, &layout_element);

        element.layout_element = LayoutElementRef::downgrade(&layout_element);
        layout_element
    }

    /// Element must have a valid LayoutElement.
    fn repopulate_element<T: 'static, R: Renderer + 'static>(element: &mut Element<T, R>) {
        let layout_element = element
            .layout_element
            .upgrade()
            .expect("Element should have a valid layout_element!");

        // The node is already attached to its parent, so nothing to re-push.
        Self::rebuild_children(element, &layout_element);
    }

    fn rebuild_children<T: 'static, R: Renderer + 'static>(
        element: &mut Element<T, R>,
        layout_element: &LayoutElementRef,
    ) {
        // Keep the old child nodes alive while we rebuild, so children that still
        // exist are reused (their `Weak` upgrades succeed) and re-attached in
        // order. Whatever isn't re-attached is freed when `old_children` drops.
        let old_children = std::mem::take(&mut layout_element.link.borrow_mut().children);

        let weak = LayoutElementRef::downgrade(layout_element);
        for child in &mut element.children {
            Self::populate_element(child, &weak);
        }

        drop(old_children);
    }

    fn resolve_clip_inner(self: &LayoutElementRef, root_clip: &LRect<f32, f32>) {
        let parent_clip = self.data().clip.clone(); // guard dropped at end of statement
        let children = self.link.borrow().children.clone(); // cheap Vec<Rc> clone

        for child in &children {
            child.calculate_clip(root_clip, &parent_clip);
            child.resolve_clip_inner(&parent_clip);
        }
    }

    #[inline]
    fn calculate_clip(&self, root_clip: &LRect<f32, f32>, parent_clip: &LRect<f32, f32>) {
        let mut data = self.data_mut();
        let style = self.style.borrow();

        let cmp_clip = if let Positioning::Pinned { flags, .. } = style.position
            && flags.contains(PinnedFlags::OVERLAY)
        {
            root_clip
        } else {
            parent_clip
        };

        let inner_x = data.rect.position.x + style.padding.left;
        let inner_y = data.rect.position.y + style.padding.top;
        let inner_w = data.rect.width() - style.padding.horizontal();
        let inner_h = data.rect.height() - style.padding.vertical();

        let inner_rect = Rect::new(inner_x, inner_y, inner_w, inner_h);

        data.clip = cmp_clip.intersection(&inner_rect).unwrap_or_default();
    }
}

#[derive(Default)]
pub(crate) struct LayoutElementLink {
    pub children: Vec<LayoutElementRef>,
    pub parent: LayoutElementWeak,
}

pub type LayoutElementRef = Rc<LayoutElement>;
pub type LayoutElementWeak = Weak<LayoutElement>;
