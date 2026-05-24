use connui::{
    context::Context,
    types::{Layout, Position, Size, SizeOp},
    widget::{Widget, WidgetId},
};
use macroquad::prelude::*;

struct Div<T = ()> {
    id: WidgetId,
    size: Size<SizeOp>,
    layout: Layout,
    children: Vec<Box<dyn Widget<T>>>,
}
impl<T> Div<T> {
    fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            id: id.into(),
            size: Size::default(),
            layout: Layout::default(),
            children: vec![],
        }
    }

    fn with_size(mut self, size: Size<SizeOp>) -> Self {
        self.size = size;

        self
    }

    fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;

        self
    }

    fn with_children(mut self, children: impl Into<Vec<Box<dyn Widget<T>>>>) -> Self {
        self.children = children.into();

        self
    }
}
impl<T> Widget<T> for Div<T> {
    #[inline]
    fn get_id(&self) -> WidgetId {
        self.id
    }

    #[inline]
    fn get_position(&self) -> Position {
        Position::Dynamic
    }

    #[inline]
    fn get_size(&self) -> Size<SizeOp> {
        self.size
    }

    #[inline]
    fn get_layout(&self) -> Layout {
        self.layout
    }

    fn get_children(&self) -> &[Box<dyn Widget<T>>] {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut [Box<dyn Widget<T>>] {
        &mut self.children
    }

    fn on_input(&self, input: ()) -> Option<connui::types::Response<T>> {
        None
    }
}

struct Gui;
impl Gui {
    fn layout(&self) -> impl Widget<()> {
        let mut root = Div::new("root").with_size(Size {
            width: SizeOp::Fit,
            height: SizeOp::Fit,
        });

        root
    }
}

#[macroquad::main("simple")]
async fn main() {
    let mut context = Context::default();
    let gui = Gui;

    loop {

        clear_background(RED);
    }
}
