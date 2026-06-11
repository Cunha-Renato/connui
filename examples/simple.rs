use connui::{prelude::*, renderer::RenderCommand};
use ggez::{
    ContextBuilder,
    event::{self, EventHandler},
    graphics::{self, Color},
};

struct Gui {
    context: Context,
    commands: Vec<RenderCommand>,
    root_width: u16,
}
impl Gui {
    fn new(dpi: f32) -> Self {
        Self {
            context: Context::default().dpi(dpi),
            commands: Default::default(),
            root_width: 240,
        }
    }

    fn layout(&self, ctx: &mut ggez::Context) -> Element<()> {
        let window_size = ctx.gfx.window().inner_size();

        let window_div = Div::default()
            .width(window_size.width as u16)
            .height(window_size.height as u16)
            .color([255, 255, 0, 255u8]);

        let root = Div::default()
            .width(self.root_width)
            .height(300)
            .padding_left(15)
            .padding_right(30)
            .horizontal()
            .color(Default::default())
            .with_children([get_big(
                true,
                [Div::new("asdas")
                    .with_color(Color::GREEN)
                    .with_size(Size {
                        width: SizeOp::Absolute(100),
                        height: SizeOp::Absolute(100),
                    })
                    .with_position(Position::Pinned {
                        position: Point { x: 300, y: 300 },
                        parent_relative: true,
                        overlay: true,
                    })
                    .into_element()],
            )])
            .into_element();

        window_div.with_children([root]).into_element()
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

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let layout = self.layout(ctx);
        let layout_result = self.context.layout(layout);

        self.commands = layout_result.render_commands;

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

fn get_fill<T: 'static>(min_w: u16, max_w: u16, color: Color) -> Element<T> {
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

fn get_big<T: 'static>(wrap: bool, children: impl IntoIterator<Item = Element<T>>) -> Element<T> {
    Div::new("big")
        .with_color(Color::MAGENTA)
        .with_size(Size {
            width: SizeOp::fill(),
            height: SizeOp::fill(),
        })
        .with_layout(Layout {
            axis: LayoutAxis::Horizontal,
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

    let dpi = ctx.gfx.window().scale_factor() as f32;

    event::run(ctx, event_loop, Gui::new(dpi));
}
