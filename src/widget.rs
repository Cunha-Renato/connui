use crate::state::State;

pub trait Widget<Response> {
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

struct ScrollView<Response> {
    children: Vec<Box<dyn Widget<Response>>>,
    offset: usize,
}
impl<Response> Widget<Response> for ScrollView<Response> {
    fn on_input(&self, input: ()) -> Option<Response> {
        None
    }

    fn children(&self) -> &[Box<dyn Widget<Response>>] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Box<dyn Widget<Response>>] {
        &mut self.children
    }

    fn update_state(&mut self, state: State) {
        if let Some(offset) = state.take() {
            self.offset = offset;
        }
    }

    fn current_state(&self) -> Option<State> {
        Some(State::new(self.offset))
    }
}
