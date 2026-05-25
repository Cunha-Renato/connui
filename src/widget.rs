use crate::{
    renderer::RenderCommand,
    state::State,
    types::{Layout, Point, Position, Response, Size, SizeOp},
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

pub trait Widget<T> {
    fn get_id(&self) -> WidgetId;

    fn get_position(&self) -> Position;

    fn get_size(&self) -> Size<SizeOp>;

    fn get_layout(&self) -> Layout;

    fn get_children(&self) -> &[Element<'_, T>];

    fn render(&self, position: Point, size: Size) -> Vec<RenderCommand>;

    fn get_current_state(&self) -> Option<State> {
        None
    }

    #[allow(unused_variables)]
    fn update_state(&mut self, state: State) {}

    //TODO: (l3gion) () is just a placeholder.
    fn on_input(&self, input: ()) -> Option<Response<T>>;

    #[inline]
    fn into_element<'a>(self) -> Element<'a, T>
    where
        Self: Sized + 'a,
    {
        Element(Box::new(self))
    }
}

pub struct Element<'a, T>(pub(crate) Box<dyn Widget<T> + 'a>);
impl<'a, T> std::ops::Deref for Element<'a, T> {
    type Target = Box<dyn Widget<T> + 'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
