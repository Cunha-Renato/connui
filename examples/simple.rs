use std::vec;

use connui::{
    context::Context,
    renderer::RenderCommand,
    types::{Layout, LayoutAxis, Position, Sides, Size, SizeOp},
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
    size: Size<SizeOp>,
    id: WidgetId,
    padding: Sides<u16>,
    margin: Sides<u16>,
    layout: Layout,
}
impl<'a, T> Div<'a, T> {
    fn new(id: impl Into<WidgetId>) -> Self {
        Self {
            children: vec![],
            color: Color::new(0., 0., 0., 1.),
            size: Size::default(),
            id: id.into(),
            padding: Sides::all(0),
            margin: Sides::all(0),
            layout: Layout::default(),
        }
    }

    fn with_size(mut self, size: Size<SizeOp>) -> Self {
        self.size = size;

        self
    }

    fn with_padding(mut self, padding: Sides<u16>) -> Self {
        self.padding = padding;
        self
    }

    fn with_margin(mut self, margin: Sides<u16>) -> Self {
        self.margin = margin;
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
    fn get_padding(&self) -> Sides<u16> {
        self.padding
    }

    #[inline]
    fn get_margin(&self) -> Sides<u16> {
        self.margin
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
            .with_padding(Sides {
                left: 15,
                right: 15,
                ..Default::default()
            })
            .with_layout(Layout {
                axis: LayoutAxis::Horizontal,
                overflow: false,
                wrap: false,
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

fn get_fill<'a, T: 'a>(min_w: u16, max_w: u16, color: Color) -> Element<'a, T> {
    Div::new("fill")
        .with_size(Size {
            width: SizeOp::Fill {
                min: min_w,
                max: max_w,
            },
            height: SizeOp::fill(),
        })
        .with_color(color)
        .into_element()
}

fn get_big<'a, T: 'a>(
    wrap: bool,
    children: impl IntoIterator<Item = Element<'a, T>>,
) -> Element<'a, T> {
    Div::new("big")
        .with_color(Color::MAGENTA)
        .with_size(Size {
            width: SizeOp::fill(),
            height: SizeOp::fill(),
        })
        .with_layout(Layout {
            axis: LayoutAxis::Horizontal,
            overflow: false,
            wrap,
        })
        .with_children(
            (0..40)
                .map(|i| {
                    let color = if i % 2 == 0 { Color::RED } else { Color::BLUE };
                    let margin = if i % 2 == 0 { 3 } else { 23 };

                    Div::new("square")
                        .with_layout(Layout {
                            axis: LayoutAxis::Horizontal,
                            overflow: true,
                            wrap: false,
                        })
                        .with_margin(Sides {
                            left: margin,
                            right: margin,
                            ..Default::default()
                        })
                        .with_size(Size {
                            width: SizeOp::Absolute(50),
                            height: SizeOp::Absolute(50),
                        })
                        .with_color(color)
                        .into_element()
                })
                .chain(children),
        )
        .into_element()
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    event::run(ctx, event_loop, Gui::default());
}
