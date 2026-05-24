use crate::widget::Widget;

#[derive(Default)]
pub struct Context {}
impl Context {
    pub fn layout<T>(&mut self, widget: impl Widget<T>) -> Vec<T> {
        todo!();
    }
}
