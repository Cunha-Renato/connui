use crate::prelude::*;
use std::sync::Arc;

pub type FontRef = Arc<dyn FontSpecs>;

pub struct GlyphData {
    pub size: Size<u16>,
    pub advance: Point<i16>,
    pub bearing: Point<i16>,
}

pub trait FontSpecs {
    fn data(&self, text_size: u16, glyph: char) -> Option<GlyphData>;
    fn ascender(&self, text_size: u16) -> Point<i16>;
    fn descender(&self, text_size: u16) -> Point<i16>;
    fn new_line(&self, text_size: u16) -> Point<i16>;
}

#[cfg(feature = "msdf_font")]
pub mod msdf {
    use super::*;
    use msdf_font::{AtlasGlyphData, GlyphBitmapData, GlyphBounds, GlyphBuilder, ttf_parser};
    use std::{collections::HashMap, path::Path};

    #[derive(Debug)]
    pub enum FontError {
        Io(std::io::Error),
        Parse(ttf_parser::FaceParsingError),
        AtlasGen,
    }
    impl From<std::io::Error> for FontError {
        #[inline]
        fn from(value: std::io::Error) -> Self {
            Self::Io(value)
        }
    }
    impl From<ttf_parser::FaceParsingError> for FontError {
        #[inline]
        fn from(value: ttf_parser::FaceParsingError) -> Self {
            Self::Parse(value)
        }
    }

    pub struct Font {
        glyphs: HashMap<char, AtlasGlyphData>,
        msdf: GlyphBitmapData<u8, 3>,
        ascender: Point<i16>,
        descender: Point<i16>,
        new_line: Point<i16>,
        units_per_em: u16,
    }
    impl FontSpecs for Font {
        fn data(&self, text_size: u16, glyph: char) -> Option<GlyphData> {
            let scale = text_size as f32 / self.units_per_em as f32;

            let g_data = self.glyphs.get(&glyph)?;
            let data = g_data.data;
            let em_bounds = data.em_bounds;
            let size = GlyphBounds {
                min: em_bounds.min.map(|v| v as f32 * scale),
                max: em_bounds.max.map(|v| v as f32 * scale),
            }
            .size();

            let size = Size {
                width: size[0].round() as u16,
                height: size[1].round() as u16,
            };

            let advance = Point {
                x: (data.advance[0] as f32 * scale).round() as i16,
                y: (data.advance[1] as f32 * scale).round() as i16,
            };

            let bearing = Point {
                x: (data.bearing[0] as f32 * scale).round() as i16,
                y: (data.bearing[1] as f32 * scale).round() as i16,
            };

            Some(GlyphData {
                size,
                advance,
                bearing,
            })
        }

        #[inline]
        fn ascender(&self, text_size: u16) -> Point<i16> {
            self.scale_point(text_size, self.ascender)
        }

        #[inline]
        fn descender(&self, text_size: u16) -> Point<i16> {
            self.scale_point(text_size, self.descender)
        }

        #[inline]
        fn new_line(&self, text_size: u16) -> Point<i16> {
            self.scale_point(text_size, self.new_line)
        }
    }
    impl Font {
        pub fn new(
            charset: impl IntoIterator<Item = char, IntoIter: Send>,
            path: impl AsRef<Path>,
        ) -> Result<Self, FontError> {
            let bytes = std::fs::read(path)?;
            Self::from_bytes(charset, &bytes)
        }

        pub fn from_bytes(
            charset: impl IntoIterator<Item = char, IntoIter: Send>,
            bytes: &[u8],
        ) -> Result<Self, FontError> {
            let face = ttf_parser::Face::parse(bytes, 0)?;

            let units_per_em = face.units_per_em();
            let ascender = Point {
                x: face.ascender(),
                y: face.vertical_ascender().unwrap_or(0),
            };

            let descender = Point {
                x: face.descender(),
                y: face.vertical_descender().unwrap_or(0),
            };

            let line_gap = Point {
                x: face.line_gap(),
                y: face.vertical_line_gap().unwrap_or(0),
            };

            let new_line = Point {
                x: ascender.x - descender.x + line_gap.x,
                y: ascender.y - descender.y + line_gap.y,
            };

            let atlas_result = GlyphBuilder::new(&face)
                .px_range(4)
                .px_size(40)
                .build_atlas(charset);

            let mut atlas = atlas_result.atlas.ok_or(FontError::AtlasGen)?;

            let msdf = atlas.msdf(3.0, true);
            let glyphs = atlas.glyph_table;

            Ok(Self {
                glyphs,
                msdf,
                ascender,
                descender,
                new_line,
                units_per_em,
            })
        }

        fn scale_point(&self, text_size: u16, point: Point<i16>) -> Point<i16> {
            Point {
                x: (point.x as f32 * text_size as f32 / self.units_per_em as f32).round() as i16,
                y: (point.y as f32 * text_size as f32 / self.units_per_em as f32).round() as i16,
            }
        }
    }
}
