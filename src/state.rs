use std::any::{Any, TypeId};

pub struct State {
    tid: TypeId,
    state: Box<dyn Any>,
}
impl State {
    pub fn new<T: 'static>(state: T) -> Self {
        Self {
            tid: TypeId::of::<T>(),
            state: Box::new(state),
        }
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        if self.tid == TypeId::of::<T>() {
            self.state.downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        if self.tid == TypeId::of::<T>() {
            self.state.downcast_mut::<T>()
        } else {
            None
        }
    }

    pub fn take<T: 'static>(self) -> Option<T> {
        if self.tid == TypeId::of::<T>() {
            Some(*self.state.downcast::<T>().ok()?)
        } else {
            None
        }
    }
}
