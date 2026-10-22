use connui::{has_style, tree::Style, types::*};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct ScrollStyle {
    pub style: Style,
}
has_style!(ScrollStyle with {style});
