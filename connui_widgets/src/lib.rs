use connui::tree::{Style, widget};
use connui::{event::Event, renderer::Renderer};
use connui::{has_layout, prelude::*};

pub mod div;
pub use div::*;
pub mod image;
pub use image::*;
pub mod button;
pub mod scroll;
pub use scroll::*;
pub mod text;
pub use text::*;
pub mod slider;
pub use slider::*;
pub mod defaults;

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