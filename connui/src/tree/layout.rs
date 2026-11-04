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

pub struct LayoutElement(RefCell<LayoutElementData>);
impl LayoutElement {
    #[inline]
    pub fn new(style: impl Into<InnerStyle>) -> Self {
        Self(RefCell::new(LayoutElementData::new(style)))
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
    pub fn borrow(&self) -> Ref<'_, LayoutElementData> {
        self.0.borrow()
    }

    #[inline]
    pub fn borrow_mut<'a>(self: &'a LayoutElementRef) -> LayoutElementInnerRefMut<'a> {
        LayoutElementInnerRefMut::new(self)
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
            if root_layout.borrow().dirty {
                if first_pass {
                    println!("RUNNING FIRST PASS LAYOUT AT DEPTH: {depth}");
                    LayoutElement::repopulate_element(root_element);
                } else {
                    println!("RUNNING SECOND PASS LAYOUT AT DEPTH: {depth}");
                }

                let parent_layout_weak = root_layout.borrow().parent.clone();

                let parent_layout = parent_layout_weak
                    .upgrade()
                    .unwrap_or_else(|| root_layout.clone());
                let parent_bounds = parent_layout.borrow().bounds;

                if first_pass {
                    root_element.pre_layout();
                }

                crate::layout::measure(root_layout, &parent_bounds);
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

        let root_clip = self.borrow().clip.clone();
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
                .borrow()
                .children
                .iter()
                .any(|c| Rc::ptr_eq(c, &layout_element));

            if !attached {
                parent.borrow_mut().push_child(layout_element.clone());
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
        let old_children = std::mem::take(&mut layout_element.borrow_mut().children);

        let weak = LayoutElementRef::downgrade(layout_element);
        for child in &mut element.children {
            Self::populate_element(child, &weak);
        }

        drop(old_children);
    }

    fn resolve_clip_inner(self: &LayoutElementRef, root_clip: &LRect<f32, f32>) {
        let borrow = self.borrow();
        let clip = &borrow.clip; // guard dropped at end of statement

        for child in &borrow.children {
            child.calculate_clip(root_clip, clip);
            child.resolve_clip_inner(clip);
        }
    }

    #[inline]
    fn calculate_clip(
        self: &LayoutElementRef,
        root_clip: &LRect<f32, f32>,
        parent_clip: &LRect<f32, f32>,
    ) {
        let mut borrow = self.borrow_mut();

        let cmp_clip = if let Positioning::Pinned { flags, .. } = borrow.style.position
            && flags.contains(PinnedFlags::OVERLAY)
        {
            root_clip
        } else {
            parent_clip
        };

        let inner_x = borrow.rect.position.x + borrow.style.padding.left;
        let inner_y = borrow.rect.position.y + borrow.style.padding.top;
        let inner_w = borrow.rect.width() - borrow.style.padding.horizontal();
        let inner_h = borrow.rect.height() - borrow.style.padding.vertical();

        let inner_rect = Rect::new(inner_x, inner_y, inner_w, inner_h);

        borrow.clip = cmp_clip.intersection(&inner_rect).unwrap_or_default();
    }

    #[inline]
    pub(crate) fn new_root() -> LayoutElementRef {
        Self::new_ref(Style::default())
    }
}

pub struct LayoutElementInnerRefMut<'a> {
    data: RefMut<'a, LayoutElementData>,
    weak: LayoutElementWeak,
}
impl<'a> LayoutElementInnerRefMut<'a> {
    fn new(layout_element: &'a LayoutElementRef) -> Self {
        Self {
            data: layout_element.0.borrow_mut(),
            weak: LayoutElementRef::downgrade(layout_element),
        }
    }

    pub fn push_child(&mut self, child: LayoutElementRef) {
        child.borrow_mut().parent = self.weak.clone();
        self.data.children.push(child);
    }

    pub fn extend_children<I: IntoIterator<Item = LayoutElementRef>>(&mut self, children: I) {
        let iter = children.into_iter().inspect(|child| {
            child.borrow_mut().parent = self.weak.clone();
        });
        self.data.children.extend(iter);
    }

    pub fn set_children(&mut self, children: Vec<LayoutElementRef>) {
        children
            .iter()
            .for_each(|child| child.borrow_mut().parent = self.weak.clone());
        self.data.children = children;
    }

    pub fn swap_children(&mut self, dst: &LayoutElementRef) {
        for this_child in &self.children {
            this_child.borrow_mut().parent = Rc::downgrade(dst);
        }

        for dst_child in &dst.borrow_mut().children {
            dst_child.borrow_mut().parent = self.weak.clone();
        }

        std::mem::swap(&mut self.children, &mut dst.borrow_mut().children);
    }
}
impl<'a> std::ops::Deref for LayoutElementInnerRefMut<'a> {
    type Target = LayoutElementData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<'a> std::ops::DerefMut for LayoutElementInnerRefMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub struct LayoutElementData {
    pub(crate) style: InnerStyle,
    pub children: Vec<LayoutElementRef>,
    pub parent: LayoutElementWeak,
    pub bounds: Bounds,
    pub rect: LRect<f32, f32>,
    pub clip: LRect<f32, f32>,
    dirty: bool,
}
impl LayoutElementData {
    #[inline]
    fn new(style: impl Into<InnerStyle>) -> Self {
        Self {
            style: style.into(),
            children: vec![],
            parent: LayoutElementWeak::new(),
            bounds: Bounds::default(),
            rect: Rect::default(),
            clip: Rect::default(),
            dirty: true,
        }
    }

    #[inline]
    pub fn style(&mut self, f: impl Fn(Style) -> Style) {
        let new_style = f(InnerStyle::to_style(&self.style)).into();

        if self.style != new_style {
            self.mark_dirty();
        }

        self.style = new_style;
    }

    pub fn children(&self) -> &[LayoutElementRef] {
        &self.children
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;

        // Parent.
        if let Some(parent) = self.parent.upgrade()
            && (!self.style.size.width.is_absolute() || !self.style.size.height.is_absolute())
            && !self.style.position.is_pinned()
        {
            let mut parent_borrow_mut = parent.borrow_mut();
            if !parent_borrow_mut.dirty {
                parent_borrow_mut.mark_dirty();
            }
        }
    }

    pub(crate) fn clear(&mut self) {
        self.dirty = false;
        self.bounds = Bounds::default();
        self.rect = Rect::default();
        self.clip = Rect::default();
    }
}

pub type LayoutElementRef = Rc<LayoutElement>;
pub type LayoutElementWeak = Weak<LayoutElement>;
