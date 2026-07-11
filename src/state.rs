use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
};

use crate::types::Id;

#[derive(Default)]
pub struct StateContext {
    states: HashMap<Id, State>,
}
impl StateContext {
    #[inline]
    pub fn entry(&mut self, id: Id) -> Entry<'_, Id, State> {
        self.states.entry(id)
    }

    #[inline]
    pub fn get<T: 'static>(&self, id: &Id) -> Option<&T> {
        self.states.get(id).and_then(State::get)
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self, id: &Id) -> Option<&mut T> {
        self.states.get_mut(id).and_then(State::get_mut)
    }

    #[inline]
    pub fn set<T: 'static>(&mut self, id: Id, state: T) -> Option<T> {
        self.states
            .insert(id, State::new(state))
            .and_then(State::take)
    }
}

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
