use crate::*;
use connui::{font, renderer::Renderer};

pub struct Text {
    layout: font::Layout,
    style: Style,
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
            style: Style::default(),
            layout,
            size: Size::default(),
        }
    }
}
impl<T, R: Renderer> Widget<T, R> for Text {
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    fn get_position(&self) -> Position {
        self.style.position
    }

    fn get_padding(&self) -> Sides<LogicalPixel> {
        self.style.padding
    }

    fn get_margin(&self) -> Sides<LogicalPixel> {
        self.style.margin
    }

    fn get_layout(&self) -> Layout {
        self.style.layout
    }

    #[inline]
    fn render(&mut self, rect: Rect, _: Rect, renderer: &mut R, _: &mut [Node<T, R>]) {
        renderer.draw_quad(rect, Color::RED, None);
        self.layout.render(renderer, rect.position);
    }

    fn update(
        &mut self,
        rect: Rect<LogicalPixel<i16>, LogicalPixel>,
        _: &mut connui::state::StateContext<R>,
    ) -> bool {
        let old_size = self.layout.shaped_size();
        if old_size.width != rect.width() || old_size.height != rect.height() {
            match self.style.layout.axis {
                LayoutAxis::Horizontal => self
                    .layout
                    .set_bounding_box(Size::new(Some(rect.width()), None)),
                LayoutAxis::Vertical => self
                    .layout
                    .set_bounding_box(Size::new(None, Some(rect.height()))),
            };

            self.layout.shape();

            let new_size = self.layout.shaped_size();
            self.size.width = SizeOp::absolute(new_size.width);
            self.size.height = SizeOp::absolute(new_size.height);

            return true;
        }

        false
    }
}
impl<T, R: Renderer> Into<Element<T, R>> for Text {
    #[inline]
    fn into(mut self) -> Element<T, R> {
        self.layout.shape();
        let text_size = self.layout.shaped_size();

        self.size.width = SizeOp::Grow {
            min: LogicalPixel::new(0),
            max: text_size.width,
            shrink: true,
        };
        self.size.height = SizeOp::Grow {
            min: LogicalPixel::new(0),
            max: text_size.height,
            shrink: true,
        };
        Element::new(self)
    }
}
impl_has_style!(trait for Text with { style });
