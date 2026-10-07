use slotmap::{SlotMap, new_key_type};

use crate::{
    has_layout, has_margin, has_padding, has_positioning, has_sizing, renderer::Renderer,
    tree::RenderElement, types::*,
};

use super::Element;

#[derive(Default, Debug, Clone, PartialEq)]
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
                axis: LayoutAxis::Horizontal,
            },
        }
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

new_key_type! { pub struct LayoutElementKey; }

pub struct LayoutElement {
    pub(crate) style: InnerStyle,
    pub(crate) children: Vec<LayoutElementKey>,

    pub bounds: Bounds,
    pub rect: LRect<f32, f32>,
    pub clip: LRect<f32, f32>,
}
impl LayoutElement {
    #[inline]
    pub fn new(style: impl Into<InnerStyle>) -> Self {
        Self {
            style: style.into(),
            children: vec![],
            bounds: Bounds::default(),
            rect: Rect::default(),
            clip: Rect::default(),
        }
    }

    #[inline]
    pub fn render_element<R: Renderer>(&self, renderer: &mut R) -> RenderElement {
        RenderElement::new(self, renderer)
    }

    #[inline]
    fn resolve_clip(&mut self, root_clip: &LRect<f32, f32>, parent_clip: &LRect<f32, f32>) {
        let cmp_clip = if let Positioning::Pinned { flags, .. } = self.style.position
            && flags.contains(PinnedFlags::OVERLAY)
        {
            root_clip
        } else {
            parent_clip
        };

        let inner_x = self.rect.position.x + self.style.padding.left;
        let inner_y = self.rect.position.y + self.style.padding.top;
        let inner_w = self.rect.width() - self.style.padding.horizontal();
        let inner_h = self.rect.height() - self.style.padding.vertical();

        let inner_rect = Rect::new(inner_x, inner_y, inner_w, inner_h);

        self.clip = cmp_clip.intersection(&inner_rect).unwrap_or_default();
    }
}

#[derive(Default)]
pub(crate) struct LayoutElementTree {
    elements: SlotMap<LayoutElementKey, LayoutElement>,
    root: Option<LayoutElementKey>,
    dirty: bool,
}
impl LayoutElementTree {
    #[inline]
    pub(crate) fn new_element(&mut self, layout_element: LayoutElement) -> LayoutElementKey {
        self.elements.insert(layout_element)
    }

    #[inline]
    pub(crate) const fn dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn layout_root<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut Element<T, R>,
    ) {
        if self.dirty {
            // Resets layout state.
            self.init(root);

            self.layout_inner();
            root.layout(self);

            if self.dirty {
                self.layout_inner();
            }

            self.resolve_clip();
        }
    }

    fn layout_inner(&mut self) {
        if !self.dirty {
            return;
        }

        println!("Running Layout");

        self.dirty = false;

        let root_key = self.root.unwrap();

        crate::layout::measure(root_key, self, &Default::default());
        crate::layout::resolve_children_size(root_key, self);
        crate::layout::resolve_children_position(root_key, self);
    }

    /// Resets layout state, assigning every [`Element`] with a new [`LayoutElementKey`] for layout computation.
    fn init<T: 'static, R: Renderer + 'static>(&mut self, root: &mut Element<T, R>) {
        self.clear();

        let root_key = Some(self.new_element(LayoutElement::new(root.element.style())));

        self.root = root_key;
        root.layout = root_key;

        self.init_children(root);
    }

    fn init_children<T: 'static, R: Renderer + 'static>(&mut self, parent: &mut Element<T, R>) {
        for child in &mut parent.children {
            let child_key = self.new_element(LayoutElement::new(child.element.style()));

            child.layout = Some(child_key);
            self[parent.layout.unwrap()].children.push(child_key);

            self.init_children(child);
        }
    }

    /// Calculates clip [`Rect`] in logical space.
    fn resolve_clip(&mut self) {
        // First against root.
        let max_rect = Rect::new(0.0.into(), 0.0.into(), f32::MAX.into(), f32::MAX.into());
        let root_key = self.root.unwrap();
        self[root_key].resolve_clip(&max_rect, &max_rect);
        let root_clip = self[root_key].clip.clone();

        self.resolve_clip_inner(&root_clip, root_key);
    }

    fn resolve_clip_inner(&mut self, root_clip: &LRect<f32, f32>, parent_key: LayoutElementKey) {
        let parent_clip = self[parent_key].clip.clone();

        for child in self[parent_key].children.clone() {
            self[child].resolve_clip(root_clip, &parent_clip);
            self.resolve_clip_inner(root_clip, child);
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.dirty = true;
        self.elements.clear()
    }
}
impl std::ops::Index<LayoutElementKey> for LayoutElementTree {
    type Output = LayoutElement;

    #[inline]
    fn index(&self, index: LayoutElementKey) -> &Self::Output {
        &self.elements[index]
    }
}
impl std::ops::IndexMut<LayoutElementKey> for LayoutElementTree {
    #[inline]
    fn index_mut(&mut self, index: LayoutElementKey) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

pub struct LayoutContext<'a> {
    layout_tree: &'a mut LayoutElementTree,
    layout_key: LayoutElementKey,
}
impl<'a> LayoutContext<'a> {
    #[inline]
    pub fn layout_element(&self) -> &LayoutElement {
        &self.layout_tree[self.layout_key]
    }

    #[inline]
    pub fn layout_element_mut(&mut self) -> &mut LayoutElement {
        &mut self.layout_tree[self.layout_key]
    }

    #[inline]
    pub fn children(&'a self) -> ECCIter<'a> {
        ECCIter::new(self.layout_key, self.layout_tree)
    }

    #[inline]
    pub fn get_child(&'a self, idx: usize) -> Option<&'a LayoutElement> {
        self.layout_tree[self.layout_key]
            .children
            .get(idx)
            .map(|&key| &self.layout_tree[key])
    }

    pub fn extend_children<I: IntoIterator<Item = LayoutElement>>(&mut self, children: I) {
        let mut curr_children = std::mem::take(&mut self.layout_tree[self.layout_key].children);

        curr_children.extend(
            children
                .into_iter()
                .map(|el| self.layout_tree.new_element(el)),
        );
        self.layout_tree[self.layout_key].children = curr_children;
    }

    #[inline]
    pub fn set_children<I: IntoIterator<Item = LayoutElement>>(&mut self, children: I) {
        self.layout_tree[self.layout_key].children.clear();
        self.extend_children(children);
    }

    #[inline]
    pub const fn relayout(&mut self) {
        self.layout_tree.dirty();
    }

    #[inline]
    pub(crate) const fn new(
        layout_key: LayoutElementKey,
        layout_tree: &'a mut LayoutElementTree,
    ) -> Self {
        Self {
            layout_tree,
            layout_key,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LayoutContextRef<'a> {
    pub(crate) layout_tree: &'a LayoutElementTree,
    pub(crate) layout_key: LayoutElementKey,
}
impl<'a> LayoutContextRef<'a> {
    #[inline]
    pub fn layout_element(&self) -> &LayoutElement {
        &self.layout_tree[self.layout_key]
    }

    #[inline]
    pub fn children(&'a self) -> ECCIter<'a> {
        ECCIter::new(self.layout_key, self.layout_tree)
    }

    #[inline]
    pub fn get_child(&'a self, idx: usize) -> Option<&'a LayoutElement> {
        self.layout_tree[self.layout_key]
            .children
            .get(idx)
            .map(|&key| &self.layout_tree[key])
    }

    #[inline]
    pub(crate) const fn new(
        layout_key: LayoutElementKey,
        layout_tree: &'a LayoutElementTree,
    ) -> Self {
        Self {
            layout_tree,
            layout_key,
        }
    }
}

pub struct ECCIter<'a> {
    tree: &'a LayoutElementTree,
    key: LayoutElementKey,
    idx: usize,
}
impl<'a> ECCIter<'a> {
    #[inline]
    const fn new(key: LayoutElementKey, tree: &'a LayoutElementTree) -> Self {
        Self { tree, key, idx: 0 }
    }
}
impl<'a> std::iter::Iterator for ECCIter<'a> {
    type Item = &'a LayoutElement;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(child_key) = self.tree[self.key].children.get(self.idx).copied() {
            self.idx += 1;

            Some(&self.tree[child_key])
        } else {
            None
        }
    }
}
