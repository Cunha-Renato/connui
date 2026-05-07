use crate::state::State;

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

pub trait Widget<Response> {
    fn id(&self) -> WidgetId;

    //TODO: (l3gion) () is just a placeholder.
    fn on_input(&self, input: ()) -> Option<Response>;

    fn children(&self) -> &[Box<dyn Widget<Response>>];

    fn children_mut(&mut self) -> &mut [Box<dyn Widget<Response>>];

    #[allow(unused_variables)]
    fn update_state(&mut self, state: State) {}

    fn current_state(&self) -> Option<State> {
        None
    }
}
