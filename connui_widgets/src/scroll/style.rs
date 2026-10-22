use connui::{has_style, tree::Style, types::*};

#[derive(Debug, Clone, PartialEq)]
pub struct ScrollStyle {
    pub style: Style,
}
impl Default for ScrollStyle {
    fn default() -> Self {
        Self {
            style: Style::new()
                .width(Sizing::fill_default())
                .height(Sizing::fill_default())
                .vertical(),
        }
    }
}
has_style!(ScrollStyle with {style});
