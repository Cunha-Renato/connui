use crate::widget::Widget;

pub struct Context {}
impl Context {
    pub fn layout<Response>(&mut self, widget: &dyn Widget<Response>) -> Option<Response> {
        widget.on_input(())
    }
}
