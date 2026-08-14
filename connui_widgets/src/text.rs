use crate::*;
use connui::{
    font,
    layout::{LayoutElement, WidgetDesc, WidgetLayout},
    renderer::Renderer,
};

pub struct Text {
    layout: font::Layout,
    size: LSize<u16>,
}
impl Text {
    pub fn new(font: font::Font, text: impl AsRef<str>) -> Self {
        let mut layout = font.layout();
        layout.text(
            font::Align::Left,
            [(text.as_ref(), font::TextAttributes::default())],
        );
        layout.shape();

        let size = layout.shaped_size();

        Self { layout, size }
    }
}
impl<T, R: Renderer> From<Text> for Element<T, R> {
    #[inline]
    fn from(value: Text) -> Self {
        Self::new(value)
    }
}

impl WidgetDiff for Text {
    fn diff_eq(&self, other: Differ) -> bool {
        other.diff_eq(self, |a, b| {
            a.layout.shaped_size() == b.layout.shaped_size()
        })
    }
}
impl WidgetDesc for Text {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size::new(
            SizeOp::Fill {
                min: 0.into(),
                max: self.size.width,
                initial: self.size.width,
            },
            SizeOp::Fill {
                min: 0.into(),
                max: self.size.height,
                initial: self.size.height,
            },
        )
    }
}
impl WidgetLayout for Text {
    fn measure(&mut self, layout_element: &mut LayoutElement, bounds: &Bounds) {
        let size = self.get_size();
        let bounds = bounds.width(size.width).height(size.height);
        layout_element.rect.size = bounds.desired(size);
        layout_element.bounds = bounds;
    }

    fn resolve(&mut self, layout_element: &mut LayoutElement) {
        let target_size = layout_element.rect.size.map(|lp| lp.as_unsigned());

        if self.size.width != target_size.width {
            self.layout
                .set_bounding_box(Size::new(Some(target_size.width), None));

            self.layout.shape();
            let layout_size = self.layout.shaped_size();

            // self.size.width = target_size.width;
            self.size.height = layout_size.height;

            // layout_element.rect.size = self.size.map(|lp| lp.as_float());
            layout_element.dirty = true;
        }
    }
}
impl<T, R: Renderer> Widget<T, R> for Text {
    #[inline]
    fn render(
        &mut self,
        render_element: RenderElement,
        layout_element: &LayoutElement,
        renderer: &mut R,
    ) {
        renderer.draw_quad(&render_element.rect, Color::BLACK, None);
        let size = layout_element.clip.size.map(|lp| lp.as_unsigned());

        self.layout
            .set_bounding_box(Size::new(Some(size.width), None));
        self.layout.render(renderer, render_element.rect.position);
    }

    #[inline]
    fn get_children(&self) -> &[Element<T, R>] {
        &[]
    }

    #[inline]
    fn get_children_mut(&mut self) -> &mut [Element<T, R>] {
        &mut []
    }
}
