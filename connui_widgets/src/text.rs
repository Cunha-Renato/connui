use crate::*;
use connui::{font, renderer::Renderer};

pub struct Text {
    layout: font::Layout,
    text_size: Size<LogicalPixel>,
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
            text_size: Size::default(),
        }
    }
}
impl<T, R: Renderer> Widget<T, R> for Text {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size {
            width: SizeOp::Fill {
                min: 0.into(),
                max: self.text_size.width,
                portion: self.text_size.width,
                shrink: true,
            },
            height: SizeOp::Fill {
                min: 0.into(),
                max: self.text_size.height,
                portion: self.text_size.height,
                shrink: true,
            },
        }
    }

    #[inline]
    fn render(&mut self, rect: Rect, _: Rect<u32, u32>, renderer: &mut R, _: &mut [Node<T, R>]) {
        // renderer.draw_quad(rect, Color::RED, None);
        self.layout.render(renderer, rect.position);
    }

    fn update(
        &mut self,
        rect: Rect<LogicalPixel<i32>, LogicalPixel>,
        _: &mut connui::state::StateContext<R>,
    ) -> bool {
        let old_size = self.layout.shaped_size();
        if old_size.width != rect.width() || old_size.height != rect.height() {
            self.layout
                .set_bounding_box(Size::new(Some(rect.width()), None));

            self.layout.shape();

            self.text_size = self.layout.shaped_size();
            return true;
        }

        false
    }
}
impl<T, R: Renderer> Into<Element<T, R>> for Text {
    #[inline]
    fn into(mut self) -> Element<T, R> {
        self.layout.shape();
        self.text_size = self.layout.shaped_size();

        Element::new(self)
    }
}
