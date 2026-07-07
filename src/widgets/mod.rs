use crate::types::{Color, Layout, LayoutFlags, Position, Sides, Size, SizeOp};

pub mod div;
pub use div::*;
pub mod button;
pub use button::*;
pub mod scrollable;
pub use scrollable::*;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Style {
    pub size: Size<SizeOp>,
    pub padding: Sides<u16>,
    pub margin: Sides<u16>,
    pub position: Position,
    pub color: Color,
    pub layout: Layout,
}

pub trait HasLayout: Sized {
    fn layout_ref(&self) -> &Layout;
    fn layout_mut(&mut self) -> &mut Layout;

    #[inline]
    fn horizontal(mut self) -> Self {
        self.layout_mut().axis = crate::types::LayoutAxis::Horizontal;
        self
    }

    #[inline]
    fn vertical(mut self) -> Self {
        self.layout_mut().axis = crate::types::LayoutAxis::Vertical;
        self
    }

    #[inline]
    fn flags(mut self, flags: LayoutFlags) -> Self {
        self.layout_mut().flags = flags;
        self
    }

    #[inline]
    fn layout(mut self, layout: Layout) -> Self {
        *self.layout_mut() = layout;
        self
    }
}
pub trait HasColor: Sized {
    fn color_ref(&self) -> &Color;
    fn color_mut(&mut self) -> &mut Color;

    #[inline]
    fn color(mut self, color: impl Into<Color>) -> Self {
        *self.color_mut() = color.into();
        self
    }
}
pub trait HasPosition: Sized {
    fn position_ref(&self) -> &Position;
    fn position_mut(&mut self) -> &mut Position;

    #[inline]
    fn position(mut self, position: Position) -> Self {
        *self.position_mut() = position;
        self
    }
}
pub trait HasSize<T>: Sized {
    fn size_ref(&self) -> &Size<T>;
    fn size_mut(&mut self) -> &mut Size<T>;

    #[inline]
    fn width(mut self, width: impl Into<T>) -> Self {
        self.size_mut().width = width.into();
        self
    }

    #[inline]
    fn height(mut self, height: impl Into<T>) -> Self {
        self.size_mut().height = height.into();
        self
    }

    #[inline]
    fn size(mut self, size: Size<T>) -> Self {
        *self.size_mut() = size;
        self
    }
}
pub trait HasPadding<T>: Sized {
    fn padding_ref(&self) -> &Sides<T>;
    fn padding_mut(&mut self) -> &mut Sides<T>;

    #[inline]
    fn padding_left(mut self, value: impl Into<T>) -> Self {
        self.padding_mut().left = value.into();
        self
    }

    #[inline]
    fn padding_right(mut self, value: impl Into<T>) -> Self {
        self.padding_mut().right = value.into();
        self
    }

    #[inline]
    fn padding_top(mut self, value: impl Into<T>) -> Self {
        self.padding_mut().top = value.into();
        self
    }

    #[inline]
    fn padding_bottom(mut self, value: impl Into<T>) -> Self {
        self.padding_mut().bottom = value.into();
        self
    }

    #[inline]
    fn padding_x<I: Into<T> + Copy>(self, value: I) -> Self {
        self.padding_left(value).padding_right(value)
    }

    #[inline]
    fn padding_y<I: Into<T> + Copy>(self, value: I) -> Self {
        self.padding_top(value).padding_bottom(value)
    }

    #[inline]
    fn padding(mut self, padding: Sides<T>) -> Self {
        *self.padding_mut() = padding;
        self
    }
}
pub trait HasMargin<T>: Sized {
    fn margin_ref(&self) -> &Sides<T>;
    fn margin_mut(&mut self) -> &mut Sides<T>;

    #[inline]
    fn margin_left(mut self, value: impl Into<T>) -> Self {
        self.margin_mut().left = value.into();
        self
    }

    #[inline]
    fn margin_right(mut self, value: impl Into<T>) -> Self {
        self.margin_mut().right = value.into();
        self
    }

    #[inline]
    fn margin_top(mut self, value: impl Into<T>) -> Self {
        self.margin_mut().top = value.into();
        self
    }

    #[inline]
    fn margin_bottom(mut self, value: impl Into<T>) -> Self {
        self.margin_mut().bottom = value.into();
        self
    }

    #[inline]
    fn margin_x<I: Into<T> + Copy>(self, value: I) -> Self {
        self.margin_left(value).margin_right(value)
    }

    #[inline]
    fn margin_y<I: Into<T> + Copy>(self, value: I) -> Self {
        self.margin_top(value).margin_bottom(value)
    }

    #[inline]
    fn margin(mut self, margin: Sides<T>) -> Self {
        *self.margin_mut() = margin;
        self
    }
}

macro_rules! create_impl_macro {
    ($d:tt, $traitname:ty, $funcname:ident, $return:ty) => {
        ::paste::paste! {
            #[macro_export]
            macro_rules! [<impl_has_ $funcname>] {
                ($d (<$d gentrait:ty>)? $d typename:ident $d (<$d gen:ident>)? {$d ($d member:tt)+}) => {
                    ::paste::paste! {
                        impl $d (<$d gen>)? $d$traitname $d(<$d gentrait>)? for $d typename $d (<$d gen>)? {
                            #[inline]
                            fn [<$funcname _ref>](&self) -> &$d$return$d(<$d gentrait>)? {
                                &self.$d ($d member)+
                            }

                            #[inline]
                            fn [<$funcname _mut>](&mut self) -> &mut $d$return$d(<$d gentrait>)? {
                                &mut self.$d ($d member)+
                            }
                        }
                    }
                }
            }

            pub use [<impl_has_ $funcname>];
        }
    };
}

create_impl_macro!($, crate::widgets::HasColor, color, crate::types::Color);
create_impl_macro!($, crate::widgets::HasLayout, layout, crate::types::Layout);
create_impl_macro!($, crate::widgets::HasPosition, position, crate::types::Position);
create_impl_macro!($, crate::widgets::HasSize, size, crate::types::Size);
create_impl_macro!($, crate::widgets::HasMargin, margin, crate::types::Sides);
create_impl_macro!($, crate::widgets::HasPadding, padding, crate::types::Sides);

impl_has_color!(Style { color });
impl_has_layout!(Style { layout });
impl_has_position!(Style { position });
impl_has_margin!(<u16> Style { margin });
impl_has_padding!(<u16> Style { padding });
impl_has_size!(<SizeOp> Style { size });
