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

    tex_bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

pub struct Renderer {
    images: HashMap<Id, ImageHandleInner>,
    state: State,

    instance_buffer: wgpu::Buffer,
    globals_buffer: wgpu::Buffer,

    // `batches` always represents the frame currently being recorded.
    // `render()` drains it (via `mem::take`) and, because `current_batch_mut`
    // lazily re-seeds an unscissored batch the next time a draw call needs
    // one, there's no separate "start of frame" step to remember to call.
    batches: Vec<Batch>,
    instances: Vec<Instance>,

    image_queue: Vec<wgpu::BindGroup>,
    scissor_queue: Vec<Rect<u32, u32>>,

    white_texture: ImageHandleInner,
    globals_data: [f32; 2],
    globals_bind_group: wgpu::BindGroup,

    viewport: PSize,

    instance_capacity: u32,
    scale_factor: f32,
}
impl Renderer {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let globals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("connui_wgpu::Renderer::globals_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
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

        let instance_capacity: u32 = 32;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("connui_wgpu::Renderer::instance_buffer"),
            size: (instance_capacity as u64) * size_of::<Instance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let globals_data = [0.0; 2];
        let globals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("connui_wgpu::Renderer::globals_buffer"),
            contents: bytemuck::cast_slice(&globals_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let globals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("connui_wgpu::Renderer::globals_bind_group"),
            layout: &globals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buffer.as_entire_binding(),
            }],
        });

        Self {
            images: HashMap::default(),
            state: State {
                device,
                queue,
                sampler,
                pipeline,
                tex_bind_group_layout,
            },
            batches: Vec::new(),
            instances: Vec::new(),
            instance_buffer,
            instance_capacity,

            image_queue: Vec::default(),
            scissor_queue: Vec::default(),

            globals_data,
            globals_buffer,
            globals_bind_group,

            white_texture,
            scale_factor: 1.0,
            viewport: PSize::default(),
        }
    }

    /// Uploads the frame's instances and records the draw calls. `width`/
    /// `height` are the render target's size in pixels.
    pub fn render(&mut self, pass: &mut wgpu::RenderPass, viewport: PSize) {
        let batches = std::mem::take(&mut self.batches);

        self.viewport = viewport;

        let width = viewport.width.inner() as u32;
        let height = viewport.height.inner() as u32;

        if !self.instances.is_empty() && width > 0 && height > 0 {
            // Updating screen_size uniform.
            if width as f32 != self.globals_data[0] || height as f32 != self.globals_data[1] {
                self.globals_data = [width as f32, height as f32];

                self.state.queue.write_buffer(
                    &self.globals_buffer,
                    0,
                    bytemuck::cast_slice(&self.globals_data),
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

            for batch in &batches {
                let scissor = clamp_scissor(batch.scissor.as_ref(), width, height);
                if scissor.width() == 0 || scissor.height() == 0 {
                    continue;
                }
                pass.set_scissor_rect(scissor.x(), scissor.y(), scissor.width(), scissor.height());

                for draw in &batch.draws {
                    if draw.len == 0 {
                        continue;
                    }
                    // Texture BindGroup
                    pass.set_bind_group(1, &draw.bind_group, &[]);
                    pass.draw(0..6, draw.start..draw.start + draw.len);
                }
            }
        }

        // Reset for the next frame. Draw calls that arrive before the next
        // `push_scissor` will lazily re-seed an unscissored batch via
        // `current_batch_mut`, so nothing else needs to run this manually.
        self.instances.clear();
        self.image_queue.clear();
        self.scissor_queue.clear();
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

    #[inline]
    fn active_bind_group(&self) -> &wgpu::BindGroup {
        self.image_queue
            .last()
            .unwrap_or(&self.white_texture.bind_group)
    }

    /// Returns the batch instances/image-switches should currently target,
    /// creating an unscissored one if the vec is empty — which is always
    /// true on the first draw call of a frame, since `render()` drains it.
    fn current_batch_mut(&mut self) -> &mut Batch {
        if self.batches.is_empty() {
            self.batches.push(Batch {
                draws: Vec::new(),
                scissor: None,
            });
        }
        self.batches.last_mut().unwrap()
    }

    fn push_instance(&mut self, instance: Instance) {
        let bind_group = self.active_bind_group().clone();
        let start = self.instances.len() as u32;

        self.instances.push(instance);

        let batch = self.current_batch_mut();
        match batch.draws.last_mut() {
            Some(draw) if draw.bind_group == bind_group => draw.len += 1,
            _ => batch.draws.push(Draw {
                start,
                len: 1,
                bind_group,
            }),
        }
    }

    fn start_batch_if_scissor_changed(&mut self, scissor: Option<&Rect<u32, u32>>) {
        if self.batches.last().map(|b| b.scissor.as_ref()) != Some(scissor) {
            self.batches.push(Batch {
                draws: Vec::new(),
                scissor: scissor.cloned(),
            });
        }
    }
}
impl connui::renderer::Renderer for Renderer {
    type ImageHandle = ImageHandle;

    fn draw_quad(&mut self, rect: &PRect, color: Color, uv: Option<&Rect<f32, f32>>) {
        self.push_instance(Instance::new(rect, uv, color));
    }

    fn push_scissor(&mut self, rect: &Rect<u32, u32>) {
        // No parent clip yet just means "unbounded" — the eventual render
        // target size clamps it in `render()`, so there's nothing to
        // intersect against here.
        let intersection = match self.scissor_queue.last() {
            Some(parent) => rect.intersection(parent).unwrap_or(Rect::new_pos_size(
                Point::new(parent.x(), parent.y()),
                Size::default(),
            )),
            None => rect.clone(),
        };

        self.start_batch_if_scissor_changed(Some(&intersection));
        self.scissor_queue.push(intersection);
    }

    fn pop_scissor(&mut self) {
        self.scissor_queue.pop();

        let restored = self.scissor_queue.pop();
        self.start_batch_if_scissor_changed(restored.as_ref());

        if let Some(restored) = restored {
            self.scissor_queue.push(restored)
        }
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

    #[inline]
    fn viewport(&self) -> PSize {
        self.viewport
    }

    #[inline]
    fn set_viewport(&mut self, viewport: PSize) {
        self.viewport = viewport;
    }
}

/// Resolves a batch's scissor against the
/// current render target size, clamping so it's always within bounds
fn clamp_scissor(scissor: Option<&Rect<u32, u32>>, width: u32, height: u32) -> Rect<u32, u32> {
    let Some(rect) = scissor else {
        return Rect::new(0, 0, width, height);
    };

    let x = rect.x().min(width);
    let y = rect.y().min(height);
    let w = rect.width().min(width.saturating_sub(x));
    let h = rect.height().min(height.saturating_sub(y));

    Rect::new(x, y, w, h)
}

struct Batch {
    draws: Vec<Draw>,
    /// `None` means unscissored — resolved to the full render target at
    /// `render()` time, since that's the first point the target size is
    /// known.
    scissor: Option<Rect<u32, u32>>,
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

    fn new(rect: &PRect, uv: Option<&Rect<f32, f32>>, color: Color) -> Self {
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
