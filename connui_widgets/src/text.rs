use crate::*;
use connui::{
    font,
    layout::{WidgetDesc, WidgetLayout},
    renderer::Renderer,
};

pub struct Text {
    layout: font::Layout,
    size: Size<SizeOp>,
}
impl Text {
    pub fn new(font: font::Font, text: impl AsRef<str>) -> Self {
        let mut layout = font.layout();
        layout.text(
            font::Align::Left,
            [(text.as_ref(), font::TextAttributes::default())],
        );

        Self {
            layout,
            size: Size::new(
                SizeOp::Fill {
                    min: 0.into(),
                    max: u16::MAX.into(),
                    initial: u16::MAX.into(),
                },
                SizeOp::Fill {
                    min: 0.into(),
                    max: u16::MAX.into(),
                    initial: u16::MAX.into(),
                },
            ),
        }
    }
}

impl WidgetDiff for Text {
    fn diff_eq(&self, other: Differ) -> bool {
        // TODO:
        other.diff_eq(self, |a, b| false)
    }
}
impl WidgetDesc for Text {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }
}
impl WidgetLayout for Text {
    fn measure(&mut self, layout: &mut connui::layout::LayoutElement, bounds: &Bounds) {
        todo!()
    }

    fn resolve_children_size(&mut self, layout: &mut connui::layout::LayoutElement) {
        todo!()
    }

    fn resolve_children_position(&mut self, layout: &mut connui::layout::LayoutElement) {
        todo!()
    }
}
impl<T, R: Renderer> Widget<T, R> for Text {
    #[inline]
    fn render(&mut self, rect: &PRect, _: &Rect<u32, u32>, renderer: &mut R, _: &[Element<T, R>]) {
        // renderer.draw_quad(rect, Color::RED, None);
        self.layout.render(renderer, rect.position);
    }

    fn update(
        &mut self,
        rect: LRect<i32, u16>,
        _: LRect<i32, u16>,
        _: &[Element<T, R>],
        _: &mut connui::state::StateContext<R>,
    ) -> bool {
        if self.size.width.is_absolute() || self.size.height.is_absolute() {
            return false;
        }

        self.layout
            .set_bounding_box(Size::new(Some(rect.width()), Some(rect.height())));
        self.layout.shape();
        self.size = self.layout.shaped_size().map(|s| s.into());

        true
    }

    fn get_children(&self) -> &[Element<T, R>] {
        &[]
    }

    fn get_children_mut(&mut self) -> &mut [Element<T, R>] {
        &mut []
    }
}
