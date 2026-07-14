use std::{collections::HashMap, sync::Arc};

use connui::{
    event::{InputEvent, MouseButton, MouseInputEvent},
    image::Handle,
    prelude::*,
    renderer::{RenderCommand, Renderer, RendererImageHandle},
};
use ggez::{
    ContextBuilder,
    event::{self, EventHandler},
    graphics,
};

struct Gui {
    context: Context<GgEzRenderer>,
    // font: FontRef,
    commands: Vec<RenderCommand<GgEzRenderer>>,
    root_width: u16,
}
impl Gui {
    fn new(scale_factor: f32) -> Self {
        Self {
            context: Context::new(GgEzRenderer::default()).scale_factor(scale_factor),
            commands: Default::default(),
            root_width: 240,
        }
    }

    fn layout(&self, ctx: &mut ggez::Context) -> Element<(), GgEzRenderer> {
        let window_size = ctx.gfx.window().inner_size();

        let window_div = Div::default()
            .width(window_size.width as u16)
            .height(window_size.height as u16)
            .color([255, 255, 0, 255]);

        let root = Div::default()
            .width(self.root_width)
            .height(300)
            .horizontal()
            .children([Image::new(Handle::once(Id::new("ac"), || {
                connui::image::HandleKind::Bytes(include_bytes!("ac.jpg").to_vec().into())
            }))
            .into()])
            .color(0x00ff00ff)
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
        _: &mut ggez::Context,
        _: ggez::input::keyboard::KeyInput,
        _: bool,
    ) -> Result<(), ggez::GameError> {
        Ok(())
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let layout = self.layout(ctx);
        let layout_result = self.context.layout(layout);
        self.context.renderer_mut().load(ctx);

        self.commands = layout_result.render_commands;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::WHITE);
        self.context
            .renderer_mut()
            .render(ctx, &mut canvas, std::mem::take(&mut self.commands));
        canvas.finish(ctx)
    }
}

fn lv1_nest<T: 'static, R: Renderer + 'static>() -> Element<T, R> {
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

fn shrink_test<T: 'static, R: Renderer + 'static>(double: bool) -> Element<T, R> {
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

struct RImageHandle(Arc<ggez::graphics::Image>);
impl Clone for RImageHandle {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self))
    }
}
impl std::ops::Deref for RImageHandle {
    type Target = Arc<ggez::graphics::Image>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<ggez::graphics::Image> for RImageHandle {
    #[inline]
    fn from(value: ggez::graphics::Image) -> Self {
        Self(Arc::new(value))
    }
}
impl RendererImageHandle for RImageHandle {
    fn width(&self) -> u32 {
        self.0.width()
    }

    fn height(&self) -> u32 {
        self.0.height()
    }
}

#[derive(Default)]
struct GgEzRenderer {
    unloaded_images: HashMap<Id, connui::image::HandleKind<GgEzRenderer>>,
    loaded_images: HashMap<Id, RImageHandle>,
}
impl Renderer for GgEzRenderer {
    type ImageHandle = RImageHandle;

    fn load_image(&mut self, handle: connui::image::Handle<Self>) -> Option<Self::ImageHandle> {
        let id = handle.id();
        let present =
            self.unloaded_images.contains_key(&id) || self.loaded_images.contains_key(&id);

        let kind = match handle.load() {
            connui::image::HandleLoad::Once(f) if !present => f.take(),
            connui::image::HandleLoad::Always(f) => f.take(),
            _ => None,
        };

        if let Some(kind) = kind {
            self.unloaded_images.insert(id, kind);
        }

        self.loaded_images.get(&id).cloned()
    }
}
impl GgEzRenderer {
    fn render(
        &self,
        ctx: &ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
        commands: Vec<RenderCommand<Self>>,
    ) {
        let mut curr_image: Option<RImageHandle> = None;

        for command in commands {
            match command {
                RenderCommand::DrawRect { rect, uv, color } => {
                    let rect =
                        ggez::graphics::Rect::new(rect.x(), rect.y(), rect.width(), rect.height());
                    let mesh = ggez::graphics::Mesh::new_rectangle(
                        ctx,
                        ggez::graphics::DrawMode::fill(),
                        rect,
                        color.into_f32().into(),
                    )
                    .unwrap();

                    if let Some(image) = curr_image.clone() {
                        canvas.draw(image.as_ref(), ggez::graphics::DrawParam::default());
                    } else {
                        canvas.draw(&mesh, ggez::graphics::DrawParam::default());
                    }
                }
                RenderCommand::PushImage(handle) => curr_image = Some(handle.clone()),
                RenderCommand::PopImage => curr_image = None,
                _ => {}
            }
        }
    }

    fn load(&mut self, ctx: &ggez::Context) {
        for (id, kind) in std::mem::take(&mut self.unloaded_images) {
            if let Ok(image) = match kind {
                connui::image::HandleKind::Path(path) => {
                    ggez::graphics::Image::from_path(&ctx.gfx, path).map(Into::into)
                }
                connui::image::HandleKind::Bytes(bytes) => {
                    ggez::graphics::Image::from_bytes(&ctx.gfx, &bytes).map(Into::into)
                }

                connui::image::HandleKind::Gpu(image) => Ok(image),
            } {
                self.loaded_images.insert(id, image);
            }
        }
    }
}

fn main() {
    let (ctx, event_loop) = ContextBuilder::new("Simple", "").build().unwrap();

    ctx.gfx.window().set_resizable(true);
    let scale_factor = ctx.gfx.window().scale_factor() as f32;
    let gui = Gui::new(scale_factor);

    event::run(ctx, event_loop, gui);
}
