use crate::font::FontRef;
use crate::prelude::*;
use crate::renderer::RenderCommand;

struct Char {
    glyph: char,
    margin: Sides<u16>,
    size: Size<u16>,
}
impl<T: 'static> From<Char> for Element<T> {
    #[inline]
    fn from(value: Char) -> Self {
        Self::new(value)
    }
}
impl<T: 'static> Widget<T> for Char {
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

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand> {
        vec![RenderCommand::DrawRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            color: Color::from_hex(0x000000ff),
        }]
    }
}

pub struct Text<T: 'static> {
    style: Style,
    text: String,
    font: FontRef,
    children: Children<T>,
    text_size: u16,
}
impl<T: 'static> Widget<T> for Text<T> {
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
    fn get_children(&mut self) -> Vec<Element<T>> {
        match std::mem::take(&mut self.children) {
            Some(vec) => *vec,
            None => vec![],
        }
    }

    #[inline]
    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand> {
        vec![
            RenderCommand::SetFont(self.font.clone()),
            RenderCommand::DrawRect {
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
                color: Color::from_hex(0xffffffff),
            },
        ]
    }
}
impl<T: 'static> From<Text<T>> for Element<T> {
    // We do all the glyph calculation here, since it runs only once per frame & before layout.
    fn from(mut value: Text<T>) -> Self {
        let ascender = value.font.ascender(value.text_size);

        let chars = value
            .text
            .chars()
            .filter_map(|c| {
                let data = value.font.as_ref().data(value.text_size, c)?;

                Some(
                    Char {
                        glyph: c,
                        margin: Sides {
                            top: (ascender.x - data.bearing.y).max(0) as u16,
                            left: data.bearing.x.max(0) as u16,
                            right: (data.advance.x - data.size.width as i16 - data.bearing.x).max(0)
                                as u16,
                            ..Default::default()
                        },
                        size: data.size,
                    }
                    .into(),
                )
            })
            .collect::<Vec<_>>();

        value.children = Some(Box::new(chars));

        Self::new(value)
    }
}
impl<T: 'static> Text<T> {
    #[inline]
    pub fn new(font: FontRef) -> Self {
        Self {
            style: Style::default(),
            text: String::new(),
            font,
            text_size: 12,
            children: None,
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

impl_has_style!(Text<T> { style });
