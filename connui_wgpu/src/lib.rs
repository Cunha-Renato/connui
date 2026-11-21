pub mod image;

use std::collections::{HashMap, hash_map::Entry};

use ::image::EncodableLayout;
use connui::{image::*, prelude::*, renderer::*};
use wgpu::util::DeviceExt;

pub use image::*;

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    sampler: wgpu::Sampler, //TODO: Future: Allow custom samplers for images.

    globals_bind_group_layout: wgpu::BindGroupLayout,
    tex_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

enum Command {
    Batch(Vec<Draw>),
}

#[derive(Clone)]
struct ClipInfo {
    clip: Clip<u32>,
    parent: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuClip {
    position: [f32; 2],
    size: [f32; 2],
    corner_radius: [f32; 4],
    parent: u32,
    _pad: [f32; 3],
}
impl From<ClipInfo> for GpuClip {
    fn from(value: ClipInfo) -> Self {
        let rect = value.clip.rect;
        let corner_radius = value.clip.corners;

        let position = [rect.x() as f32, rect.y() as f32];
        let size = [rect.width() as f32, rect.height() as f32];
        let corner_radius = [
            corner_radius.top_left as f32,
            corner_radius.top_right as f32,
            corner_radius.bottom_left as f32,
            corner_radius.bottom_right as f32,
        ];

        Self {
            position,
            size,
            corner_radius,
            parent: value.parent,
            _pad: [0.0; 3],
        }
    }
}

pub struct WgpuRenderer {
    images: HashMap<Id, ImageHandleInner>,
    state: State,

    instance_buffer: wgpu::Buffer,
    viewport_size_buffer: wgpu::Buffer,
    clip_buffer: wgpu::Buffer,

    // `batches` always represents the frame currently being recorded.
    // `render()` drains it (via `mem::take`) and, because `current_batch_mut`
    // lazily re-seeds an uncliped batch the next time a draw call needs
    // one, there's no separate "start of frame" step to remember to call.
    commands: Vec<Command>,
    instances: Vec<Instance>,
    clips: Vec<GpuClip>,

    image_queue: Vec<wgpu::BindGroup>,
    clip_queue: Vec<(u32, ClipInfo)>,

    white_texture: ImageHandleInner,
    viewport_size: [f32; 2],
    globals_bind_group: wgpu::BindGroup,

    instance_capacity: u32,
    clip_buffer_capacity: u32,
    scale_factor: f32,
}
impl WgpuRenderer {
    const DEFAULT_INSTANCE_CAPACITY: u32 = 32;
    const DEFAULT_CLIP_BUFFER_CAPACITY: u32 = 32;

    pub fn new(device: wgpu::Device, queue: wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("connui_wgpu::Renderer::globals_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("connui_wgpu::Renderer::tex_bind_group_layout"),
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
            label: Some("connui_wgpu::Renderer::linear_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline = create_pipeline(
            &device,
            format,
            &globals_bind_group_layout,
            &tex_bind_group_layout,
        );
        let white_texture = create_white_texture(&device, &queue, &tex_bind_group_layout, &sampler);

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("connui_wgpu::Renderer::instance_buffer"),
            size: (Self::DEFAULT_INSTANCE_CAPACITY as u64) * size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let viewport_size = [0.0; 2];
        let viewport_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("connui_wgpu::Renderer::viewport_size_buffer"),
            contents: bytemuck::cast_slice(&viewport_size),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let clip_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("connui_wgpu::Renderer::clip_buffer"),
            contents: &[0; Self::DEFAULT_CLIP_BUFFER_CAPACITY as usize * size_of::<GpuClip>()],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("connui_wgpu::Renderer::globals_bind_group"),
            layout: &globals_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: viewport_size_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: clip_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            images: HashMap::default(),
            state: State {
                device,
                queue,
                sampler,
                pipeline,
                tex_bind_group_layout,
                globals_bind_group_layout,
            },
            commands: Vec::new(),
            instances: Vec::new(),
            clips: Vec::new(),
            instance_buffer,
            instance_capacity: Self::DEFAULT_INSTANCE_CAPACITY,
            clip_buffer_capacity: Self::DEFAULT_CLIP_BUFFER_CAPACITY,

            image_queue: Vec::default(),
            clip_queue: Vec::default(),

            viewport_size,
            viewport_size_buffer,
            clip_buffer,
            globals_bind_group,

            white_texture,
            scale_factor: 1.0,
        }
    }

    /// Uploads the frame's instances and records the draw calls. `width`/
    /// `height` are the render target's size in pixels.
    pub fn render(&mut self, pass: &mut wgpu::RenderPass, viewport: PSize) {
        let commands = std::mem::take(&mut self.commands);
        let clips = std::mem::take(&mut self.clips);

        let width = viewport.width.inner() as u32;
        let height = viewport.height.inner() as u32;

        if !self.instances.is_empty() && width > 0 && height > 0 {
            // Updating clip buffer.
            self.ensure_clip_buffer_capacity(clips.len() as u32);
            self.state
                .queue
                .write_buffer(&self.clip_buffer, 0, bytemuck::cast_slice(&clips));

            // Updating screen_size uniform.
            if width as f32 != self.viewport_size[0] || height as f32 != self.viewport_size[1] {
                self.viewport_size = [width as f32, height as f32];

                self.state.queue.write_buffer(
                    &self.viewport_size_buffer,
                    0,
                    bytemuck::cast_slice(&self.viewport_size),
                );
            }

            self.ensure_instance_capacity(self.instances.len() as u32);

            self.state.queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instances),
            );
            self.state.queue.submit([]);

            pass.set_pipeline(&self.state.pipeline);
            pass.set_bind_group(0, &self.globals_bind_group, &[]);
            pass.set_vertex_buffer(0, self.instance_buffer.slice(..));

            for command in commands {
                match command {
                    Command::Batch(draws) => {
                        pass.set_pipeline(&self.state.pipeline);

                        for draw in draws {
                            if draw.len == 0 {
                                continue;
                            }
                            // Texture BindGroup
                            pass.set_bind_group(1, &draw.bind_group, &[]);
                            pass.draw(0..6, draw.start..draw.start + draw.len);
                        }
                    }
                }
            }
        }

        // Reset for the next frame. Draw calls that arrive before the next
        // `push_clip` will lazily re-seed an uncliped batch via
        // `current_batch_mut`, so nothing else needs to run this manually.
        self.instances.clear();
        self.image_queue.clear();
        self.clip_queue.clear();
        self.clips.clear();
    }

    fn ensure_instance_capacity(&mut self, needed: u32) {
        if needed <= self.instance_capacity {
            return;
        }

        let new_capacity = needed.next_power_of_two();
        self.instance_buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("connui_wgpu::Renderer::instance_buffer"),
            size: (new_capacity as u64) * size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = new_capacity;
    }

    fn ensure_clip_buffer_capacity(&mut self, needed: u32) {
        if needed <= self.clip_buffer_capacity {
            return;
        }
        let new_capacity = needed.next_power_of_two();
        self.clip_buffer = self.state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("connui_wgpu::Renderer::clip_buffer"),
            size: new_capacity as u64 * size_of::<GpuClip>() as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.clip_buffer_capacity = new_capacity;
        self.globals_bind_group = self
            .state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("connui_wgpu::Renderer::globals_bind_group"),
                layout: &self.state.globals_bind_group_layout, // add this field to State
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.viewport_size_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.clip_buffer.as_entire_binding(),
                    },
                ],
            });
    }

    #[inline]
    fn active_bind_group(&self) -> &wgpu::BindGroup {
        self.image_queue
            .last()
            .unwrap_or(&self.white_texture.bind_group)
    }

    fn push_instance(&mut self, instance: Instance) {
        let bind_group = self.active_bind_group().clone();
        let start = self.instances.len() as u32;

        self.instances.push(instance);

        // Prev command was draw.
        if let Some(Command::Batch(draws)) = self.commands.last_mut() {
            match draws.last_mut() {
                Some(draw) if draw.bind_group == bind_group => draw.len += 1,
                _ => draws.push(Draw {
                    start,
                    len: 1,
                    bind_group,
                }),
            }
        } else {
            self.commands.push(Command::Batch(vec![Draw {
                start,
                len: 1,
                bind_group,
            }]));
        };
    }
}
impl connui::renderer::Renderer for WgpuRenderer {
    type ImageHandle = ImageHandle;

    fn draw_quad(
        &mut self,
        rect: &PRect,
        corner_radius: &PCorners,
        color: Color,
        uv: Option<&Rect<f32, f32>>,
    ) {
        self.push_instance(Instance::new(
            rect,
            corner_radius,
            uv,
            color,
            self.clip_queue.last().map(|c| c.0).unwrap_or_default(),
        ));
    }

    fn push_clip(&mut self, clip: &Clip<u32>) {
        let parent = self.clip_queue.last().map(|c| c.0).unwrap_or_default();

        let clip_info = ClipInfo {
            clip: clip.clone(),
            parent,
        };

        let idx = self.clips.len() as u32;
        self.clip_queue.push((idx, clip_info.clone()));
        self.clips.push(clip_info.into());
    }

    fn pop_clip(&mut self) {
        self.clip_queue.pop();
    }

    fn push_image(&mut self, id: Id) {
        if let Some(image) = self.images.get(&id) {
            self.image_queue.push(image.bind_group.clone());
        }
    }

    fn pop_image(&mut self) {
        self.image_queue.pop();
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
                    write_image(
                        &self.state.queue,
                        &cached.handle,
                        Rect::new(0, 0, rgba.width(), rgba.height()),
                        4,
                        rgba.as_bytes(),
                    );
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
                rgba.width(),
                rgba.height(),
                rgba.as_bytes(),
            ),
            ImageType::Gpu(gpu) => gpu,
        };

        let bind_group = create_texture_bind_group(
            handle.id(),
            &self.state.device,
            &self.state.tex_bind_group_layout,
            &new_handle,
            &self.state.sampler,
        );

        entry.insert_entry(ImageHandleInner {
            handle: new_handle.clone(),
            bind_group,
        });

        Some(new_handle)
    }

    fn write_image<'a>(
        &mut self,
        id: &Id,
        write_op: connui::image::WriteOp<'a>,
        fallback: Option<Handle<Self>>,
    ) {
        if !self.images.contains_key(id)
            && let Some(fallback) = fallback
        {
            self.load_image(fallback);
        }

        if let Some(image) = self.images.get(id) {
            write_image(
                &self.state.queue,
                &image.handle,
                write_op.rect,
                4,
                write_op.bytes,
            );
        } else {
            let err_msg = format!("connui_wgpu::write_image::{:#?}, was not loaded!", id);
            eprintln!("{err_msg}");
        }
    }

    #[inline]
    fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    #[inline]
    fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }
}

struct Draw {
    /// Index range into `Renderer::instances` this draw covers.
    start: u32,
    len: u32,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Instance {
    position: [f32; 2],
    size: [f32; 2],
    uv: [f32; 4],
    color: [f32; 4],
    corner_radius: [f32; 4],
    clip_idx: u32,
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
            4 => Float32x4,
            5 => Uint32
        ],
    };

    fn new(
        rect: &PRect,
        corner_radius: &PCorners,
        uv: Option<&Rect<f32, f32>>,
        color: Color,
        clip_idx: u32,
    ) -> Self {
        let uv = if let Some(uv) = uv {
            [uv.x(), uv.y(), uv.width(), uv.height()]
        } else {
            [0.0, 0.0, 1.0, 1.0]
        };

        Self {
            position: [rect.x().inner(), rect.y().inner()],
            size: [rect.width().inner(), rect.height().inner()],
            uv,
            color: color.into_f32(),
            corner_radius: [
                corner_radius.top_left.inner(),
                corner_radius.top_right.inner(),
                corner_radius.bottom_left.inner(),
                corner_radius.bottom_right.inner(),
            ],
            clip_idx,
        }
    }
}

fn create_texture_bind_group(
    id: Id,
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::Texture,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(&format!(
            "connui_wgpu::Renderer::image_bind_group::{:?}",
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
    format: wgpu::TextureFormat,
    globals_bind_group_layout: &wgpu::BindGroupLayout,
    tex_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("connui_wgpu::Renderer::main_pipeline_layout"),
        bind_group_layouts: &[Some(globals_bind_group_layout), Some(tex_bind_group_layout)],
        immediate_size: 0,
    });

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("connui_wgpu::Renderer::shader_module"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("connui_wgpu::Renderer::pipeline"),
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
                format,
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
