use crate::{
    event::Event,
    renderer::Renderer,
    types::{Color, Layout, LayoutFlags, Position, Response, Sides, Size, SizeOp},
    widget::Element,
};

pub mod div;
pub use div::*;
pub mod image;
pub use image::*;
pub mod button;
pub mod scrollable;
pub mod text;

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Style {
    pub size: Size<SizeOp>,
    pub padding: Sides<u16>,
    pub margin: Sides<u16>,
    pub position: Position,
    pub color: Color,
    pub layout: Layout,
}

pub struct EventFn<T>(Box<dyn Fn(Event) -> Response<T>>);
impl<T, F: Fn(Event) -> Response<T> + 'static> From<F> for EventFn<T> {
    #[inline]
    fn from(value: F) -> Self {
        Self(Box::new(value))
    }
}
impl<T> std::ops::Deref for EventFn<T> {
    type Target = Box<dyn Fn(Event) -> Response<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
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

pub trait HasStyle: Sized {
    fn style_ref(&self) -> &Style;
    fn style_mut(&mut self) -> &mut Style;

    #[inline]
    fn style(mut self, style: Style) -> Self {
        *self.style_mut() = style;
        self
    }
}
impl<S: HasStyle> HasLayout for S {
    #[inline]
    fn layout_ref(&self) -> &Layout {
        self.style_ref().layout_ref()
    }

    #[inline]
    fn layout_mut(&mut self) -> &mut Layout {
        self.style_mut().layout_mut()
    }
}
impl<S: HasStyle> HasColor for S {
    #[inline]
    fn color_ref(&self) -> &Color {
        self.style_ref().color_ref()
    }

    #[inline]
    fn color_mut(&mut self) -> &mut Color {
        self.style_mut().color_mut()
    }
}
impl<S: HasStyle> HasPosition for S {
    #[inline]
    fn position_ref(&self) -> &Position {
        self.style_ref().position_ref()
    }

    #[inline]
    fn position_mut(&mut self) -> &mut Position {
        self.style_mut().position_mut()
    }
}
impl<S: HasStyle> HasSize<SizeOp> for S {
    #[inline]
    fn size_ref(&self) -> &Size<SizeOp> {
        self.style_ref().size_ref()
    }

    #[inline]
    fn size_mut(&mut self) -> &mut Size<SizeOp> {
        self.style_mut().size_mut()
    }
}
impl<S: HasStyle> HasPadding<u16> for S {
    #[inline]
    fn padding_ref(&self) -> &Sides<u16> {
        self.style_ref().padding_ref()
    }

    #[inline]
    fn padding_mut(&mut self) -> &mut Sides<u16> {
        self.style_mut().padding_mut()
    }
}
impl<S: HasStyle> HasMargin<u16> for S {
    #[inline]
    fn margin_ref(&self) -> &Sides<u16> {
        self.style_ref().margin_ref()
    }

    #[inline]
    fn margin_mut(&mut self) -> &mut Sides<u16> {
        self.style_mut().margin_mut()
    }
}

pub type Children<T, R> = Box<Vec<Element<T, R>>>;
pub trait HasChildren<T, R: Renderer>: Sized {
    fn children_ref(&self) -> &Children<T, R>;
    fn children_mut(&mut self) -> &mut Children<T, R>;

    #[inline]
    fn child(mut self, child: impl Into<Element<T, R>>) -> Self {
        self.children_mut().push(child.into());
        self
    }

    #[inline]
    fn children(mut self, children: impl Into<Vec<Element<T, R>>>) -> Self {
        *self.children_mut().as_mut() = children.into();
        self
    }

    #[inline]
    fn children_extend(
        mut self,
        children_iter: impl IntoIterator<Item = impl Into<Element<T, R>>>,
    ) -> Self {
        self.children_mut()
            .extend(children_iter.into_iter().map(Into::into));

        self
    }
}

macro_rules! create_impl_macro {
    ($d:tt, $traitname:ty, $funcname:ident, $return:ty) => {
        ::paste::paste! {
            #[macro_export]
            macro_rules! [<impl_has_ $funcname>] {
                ($d ({$d($d generics:tt)+})? trait $d ({$d($d trait_generics:tt)+})? for $d typename:ident $d ({$d($d type_generics:tt)+})? with {$d ($d member:tt)+}) => {
                    ::paste::paste! {
                        impl $d (<$d($d generics)+>)? $d $traitname $d(<$d($d trait_generics)+>)? for $d typename $d (<$d($d type_generics)+>)? {
                            #[inline]
                            fn [<$funcname _ref>](&self) -> &$d$return $d(<$d($d trait_generics)+>)? {
                                &self.$d ($d member)+
                            }

                            #[inline]
                            fn [<$funcname _mut>](&mut self) -> &mut $d$return $d(<$d($d trait_generics)+>)? {
                                &mut self.$d ($d member)+
                            }
                        }
                    }
                };

                ($d ({$d($d generics:tt)+})? trait $d ({$d($d trait_generics:tt)+})? for $d typename:ident $d ({$d($d type_generics:tt)+})? by {$d ($d member:tt)+}) => {
                    ::paste::paste! {
                        impl $d (<$d($d generics)+>)? $d $traitname $d(<$d($d trait_generics)+>)? for $d typename $d (<$d($d type_generics)+>)? {
                            #[inline]
                            fn [<$funcname _ref>](&self) -> &$d$return $d(<$d($d trait_generics)+>)? {
                                self.$d ($d member)+.[<$funcname _ref>]()
                            }

                            #[inline]
                            fn [<$funcname _mut>](&mut self) -> &mut $d$return $d(<$d($d trait_generics)+>)? {
                                self.$d ($d member)+.[<$funcname _mut>]()
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
create_impl_macro!($, crate::widgets::HasStyle, style, crate::widgets::Style);
create_impl_macro!($, crate::widgets::HasChildren, children, crate::widgets::Children);

impl_has_color!(trait for Style with { color });
impl_has_layout!(trait for Style with { layout });
impl_has_position!(trait for Style with { position });
impl_has_margin!(trait {u16} for Style with { margin });
impl_has_padding!(trait {u16} for Style with { padding });
impl_has_size!(trait {SizeOp} for Style with { size });
