use std::vec;

use connui::{
    context::Context,
    renderer::RenderCommand,
    types::{Layout, LayoutAxis, Position, Size, SizeOp},
    widget::{Element, Widget, WidgetId},
};
use ggez::{
    ContextBuilder,
    event::{self, EventHandler},
    graphics::{self, Color},
};

struct Div<'a, T = ()> {
    children: Vec<Element<'a, T>>,
    color: Color,
    id: WidgetId,
    size: Size<SizeOp>,
    layout: Layout,
}
impl<'a, T> Div<'a, T> {
    fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            id: id.into(),
            size: Size::default(),
            layout: Layout::default(),
            children: vec![],
            color: Color::new(0., 0., 0., 1.),
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

    fn with_children(mut self, children: impl IntoIterator<Item = Element<'a, T>>) -> Self {
        let iter = children.into_iter();

        self.children.extend(iter);

        self
    }

    fn with_color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }
}
impl<'a, T> Widget<T> for Div<'a, T> {
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

    fn get_children(&self) -> &[Element<'a, T>] {
        &self.children
    }

    fn on_input(&self, _: ()) -> Option<connui::types::Response<T>> {
        None
    }

    fn render(
        &self,
        position: connui::types::Point,
        size: Size,
    ) -> Vec<connui::renderer::RenderCommand> {
        vec![connui::renderer::RenderCommand::DrawRect {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
            color: self.color.into(),
        }]
    }
}

#[derive(Default)]
struct Gui {
    context: Context,
    commands: Vec<RenderCommand>,
}
impl Gui {
    fn layout(&self) -> Element<'static, ()> {
        let d1 = Div::new("d1")
            .with_size(Size {
                width: SizeOp::Absolute(100),
                height: SizeOp::Absolute(100),
            })
            .with_color(Color::BLUE);

        let d2 = Div::new("d2")
            .with_size(Size {
                width: SizeOp::Absolute(50),
                height: SizeOp::Absolute(50),
            })
            .with_color(Color::GREEN);

        let root = Div::new("root")
            .with_size(Size {
                width: SizeOp::Absolute(130),
                height: SizeOp::Fit,
            })
            .with_layout(Layout {
                axis: LayoutAxis::Horizontal,
                overflow: true,
                wrap: true,
            })
            .with_children([d1.into_element(), d2.into_element()]);

        root.into_element()
    }
}
impl EventHandler for Gui {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let layout = self.layout();
        self.commands = self.context.layout(layout);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        for command in &self.commands {
            match *command {
                RenderCommand::DrawRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    let rect = ggez::graphics::Rect::new(x, y, width, height);
                    let mesh = ggez::graphics::Mesh::new_rectangle(
                        ctx,
                        ggez::graphics::DrawMode::fill(),
                        rect,
                        color.into(),
                    )?;

                    canvas.draw(&mesh, ggez::graphics::DrawParam::default());
                }
            }
        }
        canvas.finish(ctx)
    }
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    event::run(ctx, event_loop, Gui::default());
}
