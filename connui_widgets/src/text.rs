use crate::*;
use connui::{font::*, renderer::Renderer};

struct Char {
    uv: Rect,
    margin: Sides<u16>,
    size: Size<u16>,
}
impl Char {
    fn new(data: GlyphData, ascender: Point<i16>) -> Self {
        Self {
            uv: data.uv,
            margin: Sides {
                top: (ascender.x - data.bearing.y).max(0) as u16,
                left: data.bearing.x.max(0) as u16,
                right: (data.advance.x - data.size.width as i16 - data.bearing.x).max(0) as u16,
                ..Default::default()
            },
            size: data.size,
        }
    }
}
impl<T: 'static, R: Renderer + 'static> From<Char> for Element<T, R> {
    #[inline]
    fn from(value: Char) -> Self {
        Self::new(value)
    }
}
impl<T: 'static, R: Renderer> Widget<T, R> for Char {
    // Always Absolute sizing.
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        Size {
            width: self.size.width.into(),
            height: self.size.height.into(),
        }
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        self.margin
    }

    #[inline]
    fn render(&self, rect: Rect, _: Rect, renderer: &mut R, _: &[Node<T, R>]) {
        renderer.draw_quad(rect, Color::from_hex(0xffffffff), Some(self.uv));
    }
}

pub struct Text<T: 'static, R: Renderer> {
    style: Style,
    text: String,
    font: Font<R>,
    children: Children<T, R>,
    text_size: u16,
}
impl<T: 'static, R: Renderer + 'static> From<Text<T, R>> for Element<T, R> {
    // We do all the glyph calculation here, since it runs only once per frame & before layout.
    fn from(mut value: Text<T, R>) -> Self {
        let ascender = value.font.ascender(value.text_size);

        let chars = value
            .font
            .data(value.text_size, &value.text)
            .into_iter()
            .map(|data| Char::new(data, ascender).into())
            .collect::<Vec<_>>();

        value.children = Box::new(chars);

        Self::new(value)
    }
}
impl<T: 'static, R: Renderer + 'static> Widget<T, R> for Text<T, R> {
    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.style.size
    }

    #[inline]
    fn get_position(&self) -> Position {
        self.style.position
    }

    #[inline]
    fn get_padding(&self) -> Sides<u16> {
        self.style.padding
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        self.style.margin
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.style.layout
    }

    #[inline]
    fn get_children(&mut self) -> Vec<Element<T, R>> {
        std::mem::take(&mut self.children)
    }

    #[inline]
    fn render(&self, _: Rect, _: Rect, renderer: &mut R, children: &[Node<T, R>]) {
        let atlas = self.font.atlas(self.text_size);
        let atlas_id = atlas.id();

        renderer.load_image(atlas);
        renderer.push_image(atlas_id);
        for child in children {
            child.render(renderer);
        }
        renderer.pop_image();
    }
}
impl<T: 'static, R: Renderer> Text<T, R> {
    #[inline]
    pub fn new(font: Font<R>) -> Self {
        Self {
            style: Style::default(),
            text: String::new(),
            font,
            text_size: 12,
            children: Children::default(),
        }
    }

    #[inline]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    #[inline]
    pub fn text_size(mut self, text_size: u16) -> Self {
        self.text_size = text_size;
        self
    }
}

impl_has_style!({T, R: Renderer} trait for Text { T, R } with { style });
