use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};

use connui::{image::*, prelude::*, renderer::*};
use wgpu::{MultisampleState, VertexState};

pub struct WgpuImageHandle(Arc<wgpu::Texture>);
impl Clone for WgpuImageHandle {
    #[inline]
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
impl From<wgpu::Texture> for WgpuImageHandle {
    #[inline]
    fn from(texture: wgpu::Texture) -> Self {
        Self(Arc::new(texture))
    }
}
impl From<&Arc<wgpu::Texture>> for WgpuImageHandle {
    #[inline]
    fn from(texture: &Arc<wgpu::Texture>) -> Self {
        Self(Arc::clone(texture))
    }
}
impl From<Arc<wgpu::Texture>> for WgpuImageHandle {
    #[inline]
    fn from(texture: Arc<wgpu::Texture>) -> Self {
        Self(texture)
    }
}
impl RendererImageHandle for WgpuImageHandle {
    #[inline]
    fn width(&self) -> u32 {
        self.0.width()
    }

    #[inline]
    fn height(&self) -> u32 {
        self.0.height()
    }
}
impl std::ops::Deref for WgpuImageHandle {
    type Target = Arc<wgpu::Texture>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct WgpuImageHandleInner {
    handle: WgpuImageHandle,
    bind_group: wgpu::BindGroup,
}

pub struct WgpuRenderer {
    images: HashMap<Id, WgpuImageHandleInner>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sampler: wgpu::Sampler, //TODO: Future: Allow custom samplers for images.
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
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

        Self {
            images: HashMap::default(),
            device,
            queue,
            sampler,
            pipeline,
            bind_group_layout,
        }
    }
}
impl Renderer for WgpuRenderer {
    type ImageHandle = WgpuImageHandle;

    fn load_image(&mut self, handle: Handle<Self>) -> Option<Self::ImageHandle> {
        let entry = self.images.entry(handle.id());

        let kind = match handle.load() {
            HandleLoad::Once(f) => match entry {
                Entry::Vacant(_) => f.take(),
                // This means that the handle is HandleKind::Once & the entry is occupied, so we don't need to load it again.
                Entry::Occupied(entry) => return Some(entry.get().handle.clone()),
            },
            HandleLoad::Always(f) => f.take(),
        };

        let image_to_load = match kind {
            // This is fresh content to load, so we need to load it and insert it into the entry.
            Some(kind) => match kind {
                HandleKind::Path(path_buf) => todo!(),
                HandleKind::Bytes(items) => todo!(),
                HandleKind::Gpu(wgpu_image) => wgpu_image,
            },
            // This means that the callback has already been consumed, and the entry is vacant, so we can't load it.
            // TODO: Log a warning here, since this is likely a bug in the user's code.
            _ => return None,
        };

        // TODO: Write texture if dimensions are the same and a handle was already loaded, instead of creating a new bind group.
        let bind_group = create_bind_group(
            handle.id(),
            &self.device,
            &self.bind_group_layout,
            &image_to_load,
            &self.sampler,
        );

        let handle = WgpuImageHandleInner {
            handle: image_to_load.clone(),
            bind_group,
        };

        entry.insert_entry(handle);

        Some(image_to_load)
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
        vertex: VertexState {
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
        multisample: MultisampleState {
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
}
