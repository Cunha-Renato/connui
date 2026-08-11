use crate::*;
use connui::{font, renderer::Renderer};

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
impl<T, R: Renderer> Widget<T, R> for Text {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    #[inline]
    fn render(&mut self, rect: &PRect, _: &Rect<u32, u32>, renderer: &mut R, _: &mut [Node<T, R>]) {
        // renderer.draw_quad(rect, Color::RED, None);
        self.layout.render(renderer, rect.position);
    }

    fn update(
        &mut self,
        rect: LRect<i32, u16>,
        _: LRect<i32, u16>,
        _: &mut Vec<Node<T, R>>,
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
}
impl<T, R: Renderer> Into<Element<T, R>> for Text {
    #[inline]
    fn into(self) -> Element<T, R> {
        Element::new(self)
    }
}
