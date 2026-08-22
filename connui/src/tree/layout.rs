use slotmap::{SlotMap, new_key_type};

use crate::{renderer::Renderer, types::*};

use super::Element;

#[derive(Default, Debug, Clone)]
pub struct Style {
    pub size: Size<SizeOp>,
    pub padding: LSides<u16>,
    pub margin: LSides<u16>,
    pub position: Position,
    pub layout: Layout,
}

#[derive(PartialEq)]
pub(crate) struct InnerStyle {
    pub size: Size<SizeOp>,
    pub padding: LSides<f32>,
    pub margin: LSides<f32>,
    pub position: Position,
    pub layout: Layout,
}
impl From<&Style> for InnerStyle {
    #[inline]
    fn from(value: &Style) -> Self {
        Self {
            size: value.size,
            padding: value.padding.map(|p| p.as_float()),
            margin: value.margin.map(|m| m.as_float()),
            position: value.position,
            layout: value.layout,
        }
    }
}

new_key_type! { pub struct LayoutElementKey; }

pub struct LayoutElement {
    pub(crate) style: InnerStyle,
    pub children: Vec<LayoutElementKey>,

    pub bounds: Bounds,
    pub rect: LRect<f32, f32>,
    pub clip: LRect<f32, f32>,
}
impl LayoutElement {
    #[inline]
    fn new(style: impl Into<InnerStyle>) -> Self {
        Self {
            style: style.into(),
            children: vec![],
            bounds: Bounds::default(),
            rect: Rect::default(),
            clip: Rect::default(),
        }
    }

    #[inline]
    fn resolve_clip(&mut self, parent_clip: &LRect<f32, f32>) {
        let inner_x = self.rect.position.x + self.style.padding.left;
        let inner_y = self.rect.position.y + self.style.padding.top;
        let inner_w = self.rect.width() - self.style.padding.get_horizontal();
        let inner_h = self.rect.height() - self.style.padding.get_vertical();

        let inner_rect = Rect::new(inner_x, inner_y, inner_w, inner_h);

        self.clip = parent_clip.intersection(&inner_rect).unwrap_or_default();
    }
}

#[derive(Default)]
pub struct LayoutElementTree {
    elements: SlotMap<LayoutElementKey, LayoutElement>,
    root: Option<LayoutElementKey>,
    dirty: bool,
}
impl LayoutElementTree {
    #[inline]
    pub fn new_element(&mut self, layout_element: LayoutElement) -> LayoutElementKey {
        self.elements.insert(layout_element)
    }

    #[inline]
    pub(crate) fn dirty(&mut self) {
        self.dirty = true;
    }

    pub(crate) fn layout<T: 'static, R: Renderer + 'static>(&mut self, root: &mut Element<T, R>) {
        if self.dirty {
            // Resets layout state.
            self.init(root);
            // Calculates layout with 5 max tries.
            self.layout_inner(root, 5);
            self.resolve_clip();
        }
    }

    fn layout_inner<T: 'static, R: Renderer + 'static>(
        &mut self,
        root: &mut Element<T, R>,
        mut max_tries: usize,
    ) {
        if !self.dirty {
            return;
        }

        if max_tries == 0 {
            eprintln!("Max tries reached in layout calculation!");

            return;
        }

        self.dirty = false;

        let root_key = self.root.unwrap();

        crate::layout::measure(root_key, self, &Default::default());
        crate::layout::resolve_children_size(root_key, self);
        crate::layout::resolve_children_position(root_key, self);

        root.layout(self);

        max_tries -= 1;
        self.layout_inner(root, max_tries);
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
        let root_key = self.root.unwrap();
        self[root_key].resolve_clip(&Rect::new(
            0.0.into(),
            0.0.into(),
            f32::MAX.into(),
            f32::MAX.into(),
        ));

        self.resolve_clip_inner(root_key);
    }

    fn resolve_clip_inner(&mut self, parent_key: LayoutElementKey) {
        let parent_clip = self[parent_key].clip.clone();

        for child in self[parent_key].children.clone() {
            self[child].resolve_clip(&parent_clip);
            self.resolve_clip_inner(child);
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
