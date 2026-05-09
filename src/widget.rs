use crate::{
    state::State,
    types::{Position, Response, Size},
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

    fn get_size(&self) -> Size;

    fn get_children(&self) -> &[Box<dyn Widget<Response<T>>>];

    fn get_children_mut(&mut self) -> &mut [Box<dyn Widget<Response<T>>>];

    fn get_current_state(&self) -> Option<State> {
        None
    }

    #[allow(unused_variables)]
    fn update_state(&mut self, state: State) {}

    //TODO: (l3gion) () is just a placeholder.
    fn on_input(&self, input: ()) -> Option<Response<T>>;
}
