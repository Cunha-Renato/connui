use crate::widget::Widget;

pub struct Context {}
impl Context {
    pub fn layout<T>(&mut self, widget: &dyn Widget<T>) -> Option<T> {
        widget.on_input(()).map(|r| match r {
            crate::types::Response::Value(v) => v,
            crate::types::Response::Callback(f) => f(),
        })
    }
}
