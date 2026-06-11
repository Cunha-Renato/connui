use crate::{
    input::InputEvent,
    renderer::RenderCommand,
    state::State,
    types::{Layout, Point, Position, Response, Sides, Size, SizeOp},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);
impl From<u64> for WidgetId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
impl From<&str> for WidgetId {
    fn from(value: &str) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }
}

pub trait Widget<T>: 'static {
    fn get_position(&self) -> Position;

    fn get_size(&self) -> Size<SizeOp>;

    #[inline]
    fn get_padding(&self) -> Sides<u16> {
        Sides::default()
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        Sides::default()
    }

    fn get_layout(&self) -> Layout;

    /// This gets called only once a frame.
    /// There should be no problem just std::mem::take the children.
    fn get_children(&mut self) -> Vec<Element<T>>;

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand>;

    #[inline]
    fn get_id(&self) -> WidgetId {
        0.into()
    }

    #[inline]
    fn get_current_state(&self) -> Option<State> {
        None
    }

    #[inline]
    #[allow(unused_variables)]
    fn update_state(&mut self, state: State) {}

    #[inline]
    #[allow(unused_variables)]
    fn on_event(&mut self, event: InputEvent) -> Response<T> {
        Default::default()
    }

    #[inline]
    fn into_element(self) -> Element<T>
    where
        Self: Sized,
    {
        Element(Box::new(self))
    }
}

pub struct Element<T>(Box<dyn Widget<T>>);
impl<T> std::ops::Deref for Element<T> {
    type Target = Box<dyn Widget<T>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> std::ops::DerefMut for Element<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
