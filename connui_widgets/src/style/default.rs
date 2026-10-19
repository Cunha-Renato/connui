use connui::types::*;

pub const GRAY_50: Color = Color::from_hex_rgb(0xFAF8F6); // #FAF8F6
pub const GRAY_100: Color = Color::from_hex_rgb(0xF3F0ED); // #F3F0ED
pub const GRAY_200: Color = Color::from_hex_rgb(0xE7E3DF); // #E7E3DF
pub const GRAY_300: Color = Color::from_hex_rgb(0xD6D0CA); // #D6D0CA
pub const GRAY_400: Color = Color::from_hex_rgb(0xB5ADA5); // #B5ADA5
pub const GRAY_500: Color = Color::from_hex_rgb(0x968D84); // #968D84
pub const GRAY_600: Color = Color::from_hex_rgb(0x776E66); // #776E66
pub const GRAY_700: Color = Color::from_hex_rgb(0x5B534C); // #5B534C
pub const GRAY_800: Color = Color::from_hex_rgb(0x3F3934); // #3F3934
pub const GRAY_900: Color = Color::from_hex_rgb(0x27231F); // #27231F

pub const ORANGE_50: Color = Color::from_hex_rgb(0xFBF4EE); // #FBF4EE
pub const ORANGE_100: Color = Color::from_hex_rgb(0xF6E5D6); // #F6E5D6
pub const ORANGE_200: Color = Color::from_hex_rgb(0xEDCDB0); // #EDCDB0
pub const ORANGE_300: Color = Color::from_hex_rgb(0xE2B088); // #E2B088
pub const ORANGE_400: Color = Color::from_hex_rgb(0xD6945F); // #D6945F
pub const ORANGE_500: Color = Color::from_hex_rgb(0xC77B42); // #C77B42
pub const ORANGE_600: Color = Color::from_hex_rgb(0xA8622F); // #A8622F
pub const ORANGE_700: Color = Color::from_hex_rgb(0x864D26); // #864D26
pub const ORANGE_800: Color = Color::from_hex_rgb(0x63391D); // #63391D
pub const ORANGE_900: Color = Color::from_hex_rgb(0x402414); // #402414

pub mod slider {
    use super::*;
    use crate::{slider::style::*, style::*};

    // STYLE
    pub const STYLE_SELECTOR: StyleSelector<'static, SliderStyle> =
        StyleSelector::new(STYLE_BASE, StyleRules::from_slice(&[]));

    pub const STYLE_BASE: SliderStyle = SliderStyle::new()
        .absolute_const(LPixel::new(130))
        .dynamic()
        .margin_const(Sides::all(LPixel::new(0)))
        .horizontal();

    // TRACK
    pub const TRACK_STYLE_SELECTOR: StyleSelector<'static, SliderTrackStyle> =
        StyleSelector::new(TRACK_STYLE_BASE, StyleRules::from_slice(TRACK_STYLE_RULES));

    pub const TRACK_STYLE_BASE: SliderTrackStyle = SliderTrackStyle::new()
        .absolute_const(LPixel::new(10))
        .color_const(ORANGE_600);

    pub const TRACK_STYLE_RULES: &[(WidgetStateFlags, Callback<SliderTrackStyle>)] = &[
        (
            WidgetStateFlags::HOVER,
            Callback::from_fn(|s| s.color_const(ORANGE_500)),
        ),
        (
            WidgetStateFlags::ACTIVE,
            Callback::from_fn(|s| s.color_const(ORANGE_400)),
        ),
        (
            WidgetStateFlags::INACTIVE,
            Callback::from_fn(|s| s.color_const(GRAY_700)),
        ),
    ];

    // TRACK RAIL
    pub const TRACK_RAIL_STYLE_SELECTOR: StyleSelector<'static, SliderTrackStyle> =
        StyleSelector::new(TRACK_RAIL_STYLE_BASE, StyleRules::from_slice(&[]));

    pub const TRACK_RAIL_STYLE_BASE: SliderTrackStyle = SliderTrackStyle::new()
        .absolute_const(LPixel::new(8))
        .color_const(GRAY_500);

    // THUMB
    pub const THUMB_STYLE_SELECTOR: StyleSelector<'static, SliderThumbStyle> =
        StyleSelector::new(THUMB_STYLE_BASE, StyleRules::from_slice(THUMB_STYLE_RULES));

    pub const THUMB_STYLE_BASE: SliderThumbStyle = SliderThumbStyle::new()
        .main_const(LPixel::new(16))
        .cross_const(LPixel::new(16))
        .color_const(ORANGE_200);

    pub const THUMB_STYLE_RULES: &[(WidgetStateFlags, Callback<SliderThumbStyle>)] = &[
        (
            WidgetStateFlags::HOVER,
            Callback::from_fn(|s| s.color_const(ORANGE_500)),
        ),
        (
            WidgetStateFlags::ACTIVE,
            Callback::from_fn(|s| s.color_const(ORANGE_400)),
        ),
        (
            WidgetStateFlags::INACTIVE,
            Callback::from_fn(|s| s.color_const(GRAY_700)),
        ),
    ];
}
