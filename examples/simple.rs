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

struct Gui {
    context: Context,
    commands: Vec<RenderCommand>,
    root_width: u16,
}
impl Default for Gui {
    fn default() -> Self {
        Self {
            context: Default::default(),
            commands: Default::default(),
            root_width: 240,
        }
    }
}
impl Gui {
    fn layout(&self) -> Element<'static, ()> {
        let root = Div::new("root")
            .with_size(Size {
                width: SizeOp::Absolute(self.root_width),
                height: SizeOp::Absolute(500),
            })
            .with_layout(Layout {
                axis: LayoutAxis::Horizontal,
                overflow: true,
                wrap: true,
            })
            .with_color(Color::BLACK)
            .with_children([get_big(true, [])]);

        root.into_element()
    }
}
impl EventHandler for Gui {
    fn mouse_wheel_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.root_width = (self.root_width as i16 + (_y * 10.0) as i16) as u16;

        Ok(())
    }

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

fn get_big<'a, T: 'a>(
    wrap: bool,
    children: impl IntoIterator<Item = Element<'a, T>>,
) -> Element<'a, T> {
    Div::new("big")
        .with_color(Color::MAGENTA)
        .with_size(Size {
            width: SizeOp::Fit,
            height: SizeOp::Fill,
        })
        .with_layout(Layout {
            axis: LayoutAxis::Horizontal,
            overflow: true,
            wrap,
        })
        .with_children(
            (0..6)
                .map(|i| {
                    let color = if i % 2 == 0 { Color::RED } else { Color::BLUE };

                    if i < 5 {
                        Div::new("square")
                            .with_layout(Layout {
                                axis: LayoutAxis::Horizontal,
                                overflow: false,
                                wrap: false,
                            })
                            .with_size(Size {
                                width: SizeOp::Absolute(50),
                                height: SizeOp::Absolute(50),
                            })
                            .with_color(color)
                            .into_element()
                    } else {
                        Div::new("square")
                            .with_layout(Layout {
                                axis: LayoutAxis::Horizontal,
                                overflow: false,
                                wrap: false,
                            })
                            .with_size(Size {
                                width: SizeOp::Fill,
                                height: SizeOp::Fill,
                            })
                            .with_color(Color::YELLOW)
                            .into_element()
                    }
                })
                .chain(children),
        )
        .into_element()
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    event::run(ctx, event_loop, Gui::default());
}
