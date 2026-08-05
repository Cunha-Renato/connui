use bitflags::bitflags;
use cosmic_text::{FontSystem, Metrics};

use crate::types::*;

pub use cosmic_text::{Align, Family, Stretch, Style, UnderlineStyle, Weight};

#[derive(Debug)]
pub struct Buffer(cosmic_text::Buffer);
impl Buffer {
    #[inline]
    pub fn set_font_size(&mut self, size: LogicalPixel) {
        let old = self.0.metrics();
        let old_mult = old.line_height / old.font_size;
        let new_size = size.inner() as f32;
        self.0
            .set_metrics(Metrics::new(new_size, new_size * old_mult));
    }

    #[inline]
    pub fn set_line_height_mult(&mut self, mult: f32) {
        let old = self.0.metrics();
        let new_line_height = old.font_size * mult;
        self.0
            .set_metrics(Metrics::new(old.font_size, new_line_height));
    }

    #[inline]
    pub fn set_metrics(&mut self, font_size: LogicalPixel, line_height_mult: f32) {
        let font_size = font_size.inner() as f32;
        self.0
            .set_metrics(Metrics::new(font_size, font_size * line_height_mult));
    }

    #[inline]
    pub fn set_bounding_box(&mut self, size: Size<Option<LogicalPixel>>) {
        self.0.set_size(
            size.width.map(|w| w.inner() as f32),
            size.height.map(|h| h.inner() as f32),
        );
    }

    #[inline]
    pub fn bounding_box(&self) -> Size<Option<LogicalPixel>> {
        let size = self.0.size();

        Size::new(
            size.0.map(|w| LogicalPixel::new(w as u16)),
            size.1.map(|h| LogicalPixel::new(h as u16)),
        )
    }

    #[inline]
    pub fn shaped_size(&mut self) -> Size<LogicalPixel> {
        let mut width = 0;
        let mut height = 0;

        for run in self.0.layout_runs() {
            width = width.max(run.line_w as u16);
            height = height.max((run.line_top + run.line_height) as u16);
        }

        Size::new(width.into(), height.into())
    }

    #[inline]
    pub fn text<'r, 's, I: IntoIterator<Item = (&'s str, TextAttributes<'r>)>>(
        &mut self,
        alignment: Align,
        text: I,
    ) {
        self.0.set_rich_text(
            text.into_iter().map(|(s, attr)| (s, attr.into())),
            &TextAttributes::default().into(),
            cosmic_text::Shaping::Advanced,
            Some(alignment),
        );
    }

    #[inline]
    pub(super) fn new(font_system: &mut FontSystem) -> Self {
        Self(cosmic_text::Buffer::new(
            font_system,
            Metrics::new(16.0, 16.0 * 1.5),
        ))
    }

    #[inline]
    pub(super) fn shape(&mut self, font_system: &mut FontSystem) {
        if self.0.redraw() {
            self.0.shape_until_scroll(font_system, true);
        }
    }

    #[inline]
    pub(super) fn layout_runs(&mut self) -> cosmic_text::LayoutRunIter<'_> {
        self.0.layout_runs()
    }
}

/// Text Attributes that defines how a given text will be displayed.
#[derive(Debug)]
pub struct TextAttributes<'a> {
    family: Family<'a>,
    decoration: TextDecoration,
    features: FontFeatures,
    weight: Weight,
    stretch: Stretch,
    color: Color,
    style: Style,
}
impl<'a> TextAttributes<'a> {
    #[inline]
    pub const fn family(mut self, family: Family<'a>) -> Self {
        self.family = family;
        self
    }

    #[inline]
    pub const fn decoration(mut self, decoration: TextDecoration) -> Self {
        self.decoration = decoration;
        self
    }

    #[inline]
    pub const fn features(mut self, features: FontFeatures) -> Self {
        self.features = features;
        self
    }

    #[inline]
    pub const fn weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }

    #[inline]
    pub const fn stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    #[inline]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[inline]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}
impl<'a> Default for TextAttributes<'a> {
    fn default() -> Self {
        Self {
            family: Family::SansSerif,
            decoration: TextDecoration::default(),
            features: FontFeatures::empty(),
            weight: Weight::NORMAL,
            stretch: Stretch::Normal,
            color: Color::WHITE,
            style: Style::Normal,
        }
    }
}
impl<'a> Into<cosmic_text::Attrs<'a>> for TextAttributes<'a> {
    #[inline]
    fn into(self) -> cosmic_text::Attrs<'a> {
        cosmic_text::Attrs {
            color_opt: Some(cosmic_text::Color(self.color.into_hex())),
            family: self.family,
            stretch: self.stretch,
            style: self.style,
            weight: self.weight,
            metadata: 0,
            cache_key_flags: cosmic_text::CacheKeyFlags::empty(),
            metrics_opt: None,
            letter_spacing_opt: None,
            font_features: self.features.into(),
            text_decoration: self.decoration.into(),
        }
    }
}

bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FontFeatures: u16 {
        const KERNING = 0b1;
        const STANDARD_LIGATURES = 0b10;
        const CONTEXTUAL_LIGATURES = 0b100;
        const CONTEXTUAL_ALTERNATES = 0b1000;
        const DISCRETIONARY_LIGATURES = 0b10000;
        const SMALL_CAPS = 0b100000;
        const ALL_SMALL_CAPS = 0b1000000;
        const STYLISTIC_SET_1 = 0b10000000;
        const STYLISTIC_SET_2 = 0b100000000;
    }
}
impl Into<cosmic_text::FontFeatures> for FontFeatures {
    fn into(self) -> cosmic_text::FontFeatures {
        use cosmic_text::FeatureTag as CTag;
        use cosmic_text::FontFeatures as CFeatures;

        let mut features = CFeatures::new();

        for feature in self.iter() {
            match feature {
                Self::KERNING => features.enable(CTag::KERNING),
                Self::STANDARD_LIGATURES => features.enable(CTag::STANDARD_LIGATURES),
                Self::CONTEXTUAL_LIGATURES => features.enable(CTag::CONTEXTUAL_LIGATURES),
                Self::CONTEXTUAL_ALTERNATES => features.enable(CTag::CONTEXTUAL_ALTERNATES),
                Self::DISCRETIONARY_LIGATURES => features.enable(CTag::DISCRETIONARY_LIGATURES),
                Self::SMALL_CAPS => features.enable(CTag::SMALL_CAPS),
                Self::ALL_SMALL_CAPS => features.enable(CTag::ALL_SMALL_CAPS),
                Self::STYLISTIC_SET_1 => features.enable(CTag::STYLISTIC_SET_1),
                Self::STYLISTIC_SET_2 => features.enable(CTag::STYLISTIC_SET_2),
                _ => &mut features,
            };
        }

        features
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextDecoration {
    underline: (UnderlineStyle, Color),
    strikethrough: (bool, Color),
    overline: (bool, Color),
}
impl TextDecoration {
    #[inline]
    pub const fn underline(mut self, style: UnderlineStyle) -> Self {
        self.underline.0 = style;
        self
    }

    #[inline]
    pub const fn underline_color(mut self, color: Color) -> Self {
        self.underline.1 = color;
        self
    }

    #[inline]
    pub const fn overline(mut self) -> Self {
        self.overline.0 = true;
        self
    }

    #[inline]
    pub const fn overline_color(mut self, color: Color) -> Self {
        self.overline.1 = color;
        self
    }

    #[inline]
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough.0 = true;
        self
    }

    #[inline]
    pub const fn strikethrough_color(mut self, color: Color) -> Self {
        self.strikethrough.1 = color;
        self
    }
}
impl Default for TextDecoration {
    #[inline]
    fn default() -> Self {
        let white = Color::WHITE;
        Self {
            underline: (UnderlineStyle::default(), white),
            strikethrough: (false, white),
            overline: (false, white),
        }
    }
}
impl Into<cosmic_text::TextDecoration> for TextDecoration {
    #[inline]
    fn into(self) -> cosmic_text::TextDecoration {
        cosmic_text::TextDecoration {
            underline: self.underline.0,
            underline_color_opt: Some(cosmic_text::Color(self.underline.1.into_hex())),
            strikethrough: self.strikethrough.0,
            strikethrough_color_opt: Some(cosmic_text::Color(self.strikethrough.1.into_hex())),
            overline: self.overline.0,
            overline_color_opt: Some(cosmic_text::Color(self.overline.1.into_hex())),
        }
    }
}
