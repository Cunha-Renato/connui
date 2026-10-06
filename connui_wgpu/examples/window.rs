use std::sync::Arc;

use connui::{prelude::*, renderer::Renderer, tree::*};
use connui_wgpu::WgpuRenderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::OwnedDisplayHandle,
    window::{Window, WindowAttributes},
};

struct AppCore<T> {
    connui_ctx: connui::context::Context<T, WgpuRenderer>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    font: connui::font::Font,
    is_surface_configured: bool,
    main_w: u16,
}
impl AppCore<()> {
    async fn new(window: Window, handle: OwnedDisplayHandle) -> Self {
        let window = Arc::new(window);
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            backend_options: wgpu::BackendOptions::default(),
            display: Some(Box::new(handle)),
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: true, // Wtf is this?
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let connui_ctx = connui::context::Context::new(WgpuRenderer::new(
            device.clone(),
            queue.clone(),
            surface_format,
        ))
        .scale_factor(window.scale_factor() as f32);

        let font = connui::font::Font::new();

        Self {
            connui_ctx,
            window,
            surface,
            config,
            device,
            queue,
            font,
            is_surface_configured: true,
            main_w: 300,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.is_surface_configured = true;
        }
    }

    fn on_ui(&mut self) -> Widget<(), WgpuRenderer> {
        use connui_widgets::*;
        let scale_factor = self.connui_ctx.renderer_mut().scale_factor();
        let window_size = self.window.inner_size();
        let viewport = Size::new(
            LPixel::new((window_size.width as f32 / scale_factor).round().max(0.0) as u16),
            LPixel::new((window_size.height as f32 / scale_factor).round().max(0.0) as u16),
        );

        // ViewPort.
        Div::default()
            .name("VIEWPORT".into())
            .horizontal()
            .color(Color::WHITE)
            .width(viewport.width)
            .height(viewport.height)
            .children_iter([
                Div::default()
                    .name("RED".into())
                    .width(200)
                    .height(200)
                    .padding(Sides::all(5))
                    .margin(Sides::all(30))
                    .color(Color::RED)
                    .children_iter([Div::default()
                        .name("BLACK".into())
                        .width(30)
                        .height(30)
                        .positioning(Positioning::Pinned {
                            position: Position::new(190.into(), 0.into()),
                            flags: PinnedFlags::all(),
                        })
                        .color(Color::BLACK)
                        .into()])
                    .into(),
                Slider::new(0.0..=f32::MAX).into(),
            ])
            .into()
    }

    fn on_message(&mut self, _: ()) {}

    fn update(&mut self) {
        let element = self.on_ui();
        for message in self.connui_ctx.layout(element) {
            self.on_message(message);
        }

        self.render();
    }

    fn render(&mut self) {
        self.window.request_redraw();

        if !self.is_surface_configured {
            return;
        }

        let output = self.surface.get_current_texture();

        match output {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => {
                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Command Encoder"),
                        });

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                        multiview_mask: None,
                    });

                    self.connui_ctx.renderer_mut().render(
                        &mut pass,
                        PSize::new(
                            PPixel::new(surface_texture.texture.width() as f32),
                            PPixel::new(surface_texture.texture.height() as f32),
                        ),
                    );
                }

                self.queue.submit(std::iter::once(encoder.finish()));
                self.queue.present(surface_texture);
            }

            _ => {}
        }
    }

    fn event(&mut self, event: WindowEvent) {
        use connui::event;

        let scale_factor = self.connui_ctx.renderer_ref().scale_factor();

        let event = match event {
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    winit::event::MouseButton::Left => event::MouseButton::LEFT,
                    winit::event::MouseButton::Right => event::MouseButton::RIGHT,
                    winit::event::MouseButton::Middle => event::MouseButton::MIDDLE,
                    winit::event::MouseButton::Back => event::MouseButton::BACKWARD,
                    winit::event::MouseButton::Forward => event::MouseButton::FORWARD,
                    winit::event::MouseButton::Other(_) => return,
                };

                event::InputEvent::Mouse(event::MouseInputEvent::Button {
                    button,
                    pressed: state.is_pressed(),
                })
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => Position::new(x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        Position::new(pos.x as f32, pos.y as f32)
                    }
                }
                .map(|d| LPixel::new(d / scale_factor).as_signed());

                self.main_w = (self.main_w as i32 + delta.y.inner() * 15).max(0) as u16;
                event::InputEvent::Mouse(event::MouseInputEvent::Scroll(delta))
            }
            WindowEvent::CursorMoved { position, .. } => {
                event::InputEvent::Mouse(event::MouseInputEvent::Move(
                    Position::new(position.x as f32, position.y as f32)
                        .map(|p| LPixel::new(p / scale_factor).as_signed()),
                ))
            }
            WindowEvent::CursorEntered { .. } => {
                event::InputEvent::Window(event::WindowEvent::CursorEnter)
            }
            WindowEvent::CursorLeft { .. } => {
                event::InputEvent::Window(event::WindowEvent::CurserLeft)
            }

            _ => return,
        };

        self.connui_ctx.event(event);
    }
}

#[derive(Default)]
struct App {
    core: Option<AppCore<()>>,
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.core = Some(pollster::block_on(AppCore::new(
            event_loop
                .create_window(WindowAttributes::default().with_title("WGPU Example"))
                .unwrap(),
            event_loop.owned_display_handle(),
        )))
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(core) = &mut self.core else {
            return;
        };

        match event {
            winit::event::WindowEvent::Resized(size) => {
                core.resize(size.width, size.height);
            }
            winit::event::WindowEvent::CloseRequested => event_loop.exit(),
            winit::event::WindowEvent::RedrawRequested => core.update(),
            _ => {
                core.event(event);
            }
        }
    }
}

fn main() {
    let el = winit::event_loop::EventLoop::new().unwrap();
    el.run_app(&mut App::default()).unwrap();
}
