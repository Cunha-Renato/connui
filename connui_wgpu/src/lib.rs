pub mod image;
use ::image::EncodableLayout;
pub use image::*;

use connui::{font::Font, image::*, prelude::*, renderer::*};
use std::collections::{HashMap, hash_map::Entry};
use wgpu::util::DeviceExt;

struct WgpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    sampler: wgpu::Sampler, //TODO: Future: Allow custom samplers for images.

    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

pub struct WgpuRenderer {
    images: HashMap<Id, WgpuImageHandleInner>,
    state: WgpuState,

    current_batch: Batch,
    batches: Vec<Batch>,
    image_queue: Vec<wgpu::BindGroup>,
    scissor_queue: Vec<Rect>,
    current_text: Option<Text>,
    white_texture: WgpuImageHandleInner,
}
impl WgpuRenderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("connui_wgpu::WgpuRenderer::bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("connui_wgpu::WgpuRenderer::linear_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline = create_pipeline(&device, &bind_group_layout);
        let white_texture = create_white_texture(&device, &queue, &bind_group_layout, &sampler);

        Self {
            images: HashMap::default(),
            state: WgpuState {
                device,
                queue,
                sampler,
                pipeline,
                bind_group_layout,
            },
            current_batch: Batch::default(),
            batches: Vec::default(),
            image_queue: Vec::default(),
            scissor_queue: Vec::default(),
            current_text: None,
            white_texture,
        }
    }

    pub fn render(&mut self, pass: &mut wgpu::RenderPass) {
        for batch in std::mem::take(&mut self.batches) {
            pass.set_scissor_rect(
                batch.scissor.x() as u32,
                batch.scissor.y() as u32,
                batch.scissor.width() as u32,
                batch.scissor.height() as u32,
            );
            pass.set_pipeline(&self.state.pipeline);

            for draw in batch.draws {
                self.dispatch(pass, draw.instances, draw.bind_group);
            }
        }
    }

    fn dispatch(
        &self,
        pass: &mut wgpu::RenderPass,
        instances: Vec<Instance>,
        bind_group: wgpu::BindGroup,
    ) {
        let instance_buffer =
            self.state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("connui_wgpu::WgpuRenderer::instance_buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        pass.set_bind_group(0, &bind_group, &[]);
        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw(0..6, 0..instances.len() as u32);
    }
}
impl Renderer for WgpuRenderer {
    type ImageHandle = WgpuImageHandle;

    fn draw_quad(&mut self, rect: Rect, color: Color, uv: Option<Rect>) {
        self.current_batch.push(
            Instance::new(rect, uv, color),
            &self.white_texture.bind_group,
        )
    }

    fn draw_char(&mut self, rect: Rect, color: Color, char: char) {
        if let Some(text) = &self.current_text {
            let uv = text.font.uv(char, text.size);

            self.draw_quad(rect, color, Some(uv));
        }
    }

    fn push_scissor(&mut self, rect: Rect) {
        let intersection = self
            .scissor_queue
            .last()
            .map(|sc| sc.intersection(&rect))
            .flatten()
            .unwrap_or(rect);

        self.scissor_queue.push(intersection);
        if self.current_batch.scissor != intersection {
            self.batches.push(Batch {
                draws: vec![],
                scissor: intersection,
            })
        }
    }

    fn pop_scissor(&mut self) {
        self.scissor_queue.pop();
    }

    fn push_image(&mut self, id: Id) {
        if let Some(image) = self.images.get(&id) {
            self.image_queue.push(image.bind_group.clone());
            self.current_batch.set_image(&image.bind_group);
        }
    }

    fn pop_image(&mut self) {
        if let Some(bind_group) = self.image_queue.pop() {
            self.current_batch.set_image(&bind_group);
        }
    }

    fn begin_text(&mut self, font: Font<Self>, text_size: u16) {
        let handle = font.atlas(text_size);
        let id = handle.id();

        // Maybe this is shit.
        self.load_image(handle);
        self.push_image(id);
        self.current_text = Some(Text {
            font,
            size: text_size,
        });
    }

    fn end_text(&mut self) {
        self.pop_image();
        self.current_text = None;
    }

    fn load_image(&mut self, handle: Handle<Self>) -> Option<Self::ImageHandle> {
        let mut entry = self.images.entry(handle.id());

        // If the image is already loaded and the handle is a run-once handle, return the existing image.
        if let Entry::Occupied(entry) = &entry
            && matches!(handle.load(), HandleLoad::Once(_))
        {
            return Some(entry.get().handle.clone());
        }

        // None here means either the callback was consumed or image loading failed.
        // TODO: Future: Consider returning a Result with an error type for image loading failures.
        let content = match handle.load().take()? {
            HandleKind::Path(path) => load_path(&path).map(ImageType::Rgba),
            HandleKind::Bytes(bytes) => load_bytes(&bytes).map(ImageType::Rgba),
            HandleKind::Custom {
                width,
                height,
                bpp,
                bytes,
            } => load_custom(width, height, bpp, &bytes).map(ImageType::Rgba),
            HandleKind::Gpu(gpu_handle) => Some(ImageType::Gpu(gpu_handle)),
        }?;

        // Same-size update: rewrite the texture in place instead of
        // allocating a new texture + bind group.
        if let Entry::Occupied(entry) = &mut entry {
            let cached = entry.get_mut();
            match &content {
                ImageType::Rgba(rgba)
                    if cached.handle.width() == rgba.width()
                        && cached.handle.height() == rgba.height() =>
                {
                    write_image(&self.state.queue, &cached.handle, rgba.as_bytes(), 4);
                    return Some(cached.handle.clone());
                }
                ImageType::Gpu(new)
                    if cached.handle.size() == new.size()
                        && cached.handle.format() == new.format() =>
                {
                    cached.handle = new.clone();
                    return Some(cached.handle.clone());
                }
                _ => {}
            }
        }

        let new_handle = match content {
            ImageType::Rgba(rgba) => create_image(
                handle.id(),
                &self.state.device,
                &self.state.queue,
                rgba.as_bytes(),
            )?,
            ImageType::Gpu(gpu) => gpu,
        };

        let bind_group = create_bind_group(
            handle.id(),
            &self.state.device,
            &self.state.bind_group_layout,
            &new_handle,
            &self.state.sampler,
        );

        entry.insert_entry(WgpuImageHandleInner {
            handle: new_handle.clone(),
            bind_group,
        });

        Some(new_handle)
    }
}

struct Batch {
    draws: Vec<Draw>,
    scissor: Rect,
}
impl Batch {
    fn push(&mut self, instance: Instance, default_image: &wgpu::BindGroup) {
        if let Some(draw) = self.draws.last_mut() {
            draw.instances.push(instance);
        } else {
            self.draws.push(Draw {
                instances: vec![instance],
                bind_group: default_image.clone(),
            });
        }
    }

    fn set_image(&mut self, bind_group: &wgpu::BindGroup) {
        if let Some(draw) = self.draws.last_mut() {
            if draw.instances.is_empty() && &draw.bind_group != bind_group {
                draw.bind_group = bind_group.clone();
                return;
            } else if &draw.bind_group == bind_group {
                return;
            }
        }

        self.draws.push(Draw {
            instances: vec![],
            bind_group: bind_group.clone(),
        })
    }
}
impl Default for Batch {
    #[inline]
    fn default() -> Self {
        Self {
            draws: Default::default(),
            scissor: Rect::new(0.0, 0.0, f32::MAX, f32::MAX),
        }
    }
}

struct Draw {
    instances: Vec<Instance>,
    bind_group: wgpu::BindGroup,
}

struct Text {
    font: Font<WgpuRenderer>,
    size: u16,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Instance {
    position: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    color: [f32; 4],
}
impl Instance {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Float32x4,
            3 => Float32x4,
        ],
    };

    fn new(rect: Rect, uv: Option<Rect>, color: Color) -> Self {
        let uv = if let Some(uv) = uv {
            [uv.x(), uv.y(), uv.width(), uv.height()]
        } else {
            [0.0, 0.0, 1.0, 1.0]
        };

        Self {
            position: [rect.x(), rect.y()],
            size: [rect.width(), rect.height()],
            uv,
            color: color.into_f32(),
        }
    }
}

fn create_bind_group(
    id: Id,
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!(
            "connui_wgpu::WgpuRenderer::image_bind_group::{:?}",
            id
        )),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn create_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("connui_wgpu::WgpuRenderer::main_pipeline_layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("connui_wgpu::WgpuRenderer::shader_module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("connui_wgpu::WgpuRenderer::pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[Some(Instance::LAYOUT)],
        },
        primitive: wgpu::PrimitiveState {
            topology: Default::default(),
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
