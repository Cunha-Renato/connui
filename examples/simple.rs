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
            .width(self.root_width)
            .height(300)
            .padding(Sides::all(20))
            .horizontal()
            .color(Color::from_hex(0x00ff00ff))
            .children([])
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

fn lv1_nest<T: 'static>() -> Element<T> {
    Div::default()
        .color(0xff0000ff)
        .padding(Sides::all(10))
        .children([
            Div::default()
                .color(0x0000ffff)
                .margin(Sides::all(3))
                .width(100)
                .height(100)
                .into(),
            Div::default()
                .color(0xffffffff)
                .margin(Sides::all(3))
                .width(110)
                .height(110)
                .into(),
        ])
        .into()
}

fn shrink_test<T: 'static>(double: bool) -> Element<T> {
    let child_size = if double { 60 } else { 30 };

    Div::default()
        .color(0xff0000ff)
        .margin(Sides::all(5))
        .horizontal()
        .children(
            (0..3)
                .map(|_| {
                    Div::default()
                        .color(0x0000ffff)
                        .width(child_size)
                        .height(child_size)
                        .margin(Sides::all(2))
                        .into()
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    let dpi = ctx.gfx.window().scale_factor() as f32;

    event::run(ctx, event_loop, Gui::new(dpi));
}
