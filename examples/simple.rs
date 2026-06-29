use connui::{
    event::{InputEvent, MouseButton, MouseEvent, MouseInputEvent},
    prelude::*,
    renderer::RenderCommand,
};
use ggez::{
    ContextBuilder,
    event::{self, EventHandler},
    graphics,
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
            .color([255, 255, 0, 255]);

        let root = Div::default()
            .width(300)
            .height(300)
            .padding_left(15u16)
            .padding_right(30u16)
            .horizontal()
            .color(Color::from_hex(0x00ff00ff))
            .children([get_scrollable((0..10).map(|_| {
                Div::default()
                    .color(Color::from_hex(0xff0000ff))
                    .margin(Sides {
                        bottom: 10,
                        ..Default::default()
                    })
                    .width(150)
                    .height(110)
                    .into()
            }))])
            .into();

        window_div.children([root]).into()
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
        self.context
            .event(InputEvent::Mouse(MouseInputEvent::Scroll(Point {
                x: _x as i16,
                y: _y as i16,
            })));

        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.context
            .event(InputEvent::Mouse(MouseInputEvent::Button {
                button: match _button {
                    event::MouseButton::Left => MouseButton::Left,
                    event::MouseButton::Right => MouseButton::Right,
                    event::MouseButton::Middle => MouseButton::Middle,
                    event::MouseButton::Other(276) => MouseButton::Forward,
                    event::MouseButton::Other(275) => MouseButton::Backwad,
                    _ => return Ok(()),
                },
                pressed: true,
            }));

        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) -> Result<(), ggez::GameError> {
        self.context
            .event(InputEvent::Mouse(MouseInputEvent::Button {
                button: match _button {
                    event::MouseButton::Left => MouseButton::Left,
                    event::MouseButton::Right => MouseButton::Right,
                    event::MouseButton::Middle => MouseButton::Middle,
                    event::MouseButton::Other(276) => MouseButton::Forward,
                    event::MouseButton::Other(275) => MouseButton::Backwad,
                    _ => return Ok(()),
                },
                pressed: false,
            }));

        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut ggez::Context,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
    ) -> Result<(), ggez::GameError> {
        self.context
            .event(InputEvent::Mouse(MouseInputEvent::Move(Point {
                x: _x as i16,
                y: _y as i16,
            })));

        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        input: ggez::input::keyboard::KeyInput,
        _repeated: bool,
    ) -> Result<(), ggez::GameError> {
        println!("Repeated: {_repeated}.");
        println!("Key: {input:#?}");

        Ok(())
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let layout = self.layout(ctx);
        let layout_result = self.context.layout(layout);

        self.commands = layout_result.render_commands;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);

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
                        color.into_f32().into(),
                    )?;

                    canvas.draw(&mesh, ggez::graphics::DrawParam::default());
                }
            }
        }
        canvas.finish(ctx)
    }
}

fn get_fill<T: 'static>(min_w: u16, max_w: u16, color: Color) -> Element<T> {
    Div::default()
        .size(Size {
            width: SizeOp::Fill {
                min: min_w,
                max: max_w,
            },
            height: SizeOp::fill(),
        })
        .color(color)
        .into()
}

fn get_big<T: 'static>(wrap: bool, children: impl IntoIterator<Item = Element<T>>) -> Element<T> {
    Div::default()
        .color([255, 110, 110, 255])
        .size(Size {
            width: SizeOp::fill(),
            height: SizeOp::fill(),
        })
        .layout(Layout {
            axis: LayoutAxis::Horizontal,
            wrap,
        })
        .children_iter(
            (0..40)
                .map(|i| {
                    let color = if i % 2 == 0 {
                        Color::from_bytes([255, 0, 0, 255])
                    } else {
                        Color::from_bytes([0, 0, 255, 255])
                    };
                    let margin = if i % 2 == 0 { 3 } else { 23 };

                    Div::default()
                        .layout(Layout {
                            axis: LayoutAxis::Horizontal,
                            wrap: false,
                        })
                        .margin(Sides {
                            left: margin,
                            right: margin,
                            ..Default::default()
                        })
                        .size(Size {
                            width: SizeOp::Absolute(50),
                            height: SizeOp::Absolute(50),
                        })
                        .color(color)
                        .into()
                })
                .chain(children),
        )
        .into()
}

fn get_scrollable<T: 'static>(children: impl IntoIterator<Item = Element<T>>) -> Element<T> {
    ScrollDiv::new("ScrollBaby")
        .width(250)
        .height(250)
        .layout(Layout {
            axis: LayoutAxis::Vertical,
            wrap: false,
        })
        .children(children.into_iter().collect::<Vec<_>>())
        .into()
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    let dpi = ctx.gfx.window().scale_factor() as f32;

    event::run(ctx, event_loop, Gui::new(dpi));
}
