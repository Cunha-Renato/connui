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
                    portion: u16::MAX.into(),
                    shrink: true,
                },
                SizeOp::Fill {
                    min: 0.into(),
                    max: u16::MAX.into(),
                    portion: u16::MAX.into(),
                    shrink: true,
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
    fn render(&mut self, rect: Rect, _: Rect<u32, u32>, renderer: &mut R, _: &mut [Node<T, R>]) {
        renderer.draw_quad(rect, Color::RED, None);
        self.layout.render(renderer, rect.position);
    }

    fn update(
        &mut self,
        _: Rect<LogicalPixel<i32>, LogicalPixel>,
        clip: Rect<LogicalPixel<i32>, LogicalPixel>,
        _: &mut connui::state::StateContext<R>,
    ) -> bool {
        self.layout
            .set_bounding_box(Size::new(Some(clip.width()), Some(clip.height())));
        self.layout.shape();
        let text_size = self.layout.shaped_size();

        self.size = Size::new(
            SizeOp::Fill {
                min: 0.into(),
                max: text_size.width,
                portion: text_size.width,
                shrink: true,
            },
            SizeOp::Fill {
                min: 0.into(),
                max: text_size.height,
                portion: text_size.height,
                shrink: true,
            },
        );

        true
    }
}
impl<T, R: Renderer> Into<Element<T, R>> for Text {
    #[inline]
    fn into(self) -> Element<T, R> {
        Element::new(self)
    }
}
