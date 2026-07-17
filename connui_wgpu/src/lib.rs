use connui::{image::*, prelude::*, renderer::*};
use image::{EncodableLayout, Pixel};
use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
};
use wgpu::util::DeviceExt;

pub struct WgpuRenderer {
    images: HashMap<Id, WgpuImageHandleInner>,
    commands: Vec<RenderCommand<Self>>,
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
            commands: Vec::default(),
            device,
            queue,
            sampler,
            pipeline,
            bind_group_layout,
        }
    }

    pub fn render(&mut self, pass: &mut wgpu::RenderPass) {
        let commands = std::mem::take(&mut self.commands);

        if commands.is_empty() {
            return;
        }

        pass.set_pipeline(&self.pipeline);

        let mut instances = Vec::with_capacity(commands.len());

        for command in commands {
            match command {
                RenderCommand::DrawRect { rect, uv, color } => {
                    let uv = if let Some(uv) = uv {
                        uv
                    } else {
                        Rect::new(0.0, 0.0, 1.0, 1.0)
                    };

                    instances.push(Instance {
                        position: [rect.x(), rect.y()],
                        size: [rect.width(), rect.height()],
                        uv: [uv.x(), uv.y(), uv.width(), uv.height()],
                        color: color.into_f32(),
                    });
                }
                RenderCommand::PushImage { id, .. } => {
                    if let Some(image) = self.images.get(&id) {
                        self.dispatch(pass, &mut instances);

                        pass.set_bind_group(0, &image.bind_group, &[]);
                    }
                }
                RenderCommand::PopImage => self.dispatch(pass, &mut instances),
            }
        }
    }

    fn dispatch(&self, pass: &mut wgpu::RenderPass, instances: &mut Vec<Instance>) {
        let instance_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("connui_wgpu::WgpuRenderer::instance_buffer"),
                contents: bytemuck::cast_slice(instances),
                usage: wgpu::BufferUsages::VERTEX,
            });

        instances.clear();

        pass.set_vertex_buffer(0, instance_buffer.slice(..));
        pass.draw(0..6, 0..instances.len() as u32);
    }
}
impl Renderer for WgpuRenderer {
    type ImageHandle = WgpuImageHandle;

    #[inline]
    fn record<I: IntoIterator<Item = RenderCommand<Self>>>(&mut self, commands: I) {
        self.commands.extend(commands.into_iter());
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
                    write_image(&self.queue, &cached.handle, rgba.as_bytes(), 4);
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
            ImageType::Rgba(rgba) => {
                create_image(handle.id(), &self.device, &self.queue, rgba.as_bytes())?
            }
            ImageType::Gpu(gpu) => gpu,
        };

        let bind_group = create_bind_group(
            handle.id(),
            &self.device,
            &self.bind_group_layout,
            &new_handle,
            &self.sampler,
        );

        entry.insert_entry(WgpuImageHandleInner {
            handle: new_handle.clone(),
            bind_group,
        });

        Some(new_handle)
    }

    #[inline]
    fn supported_font_render_method(&self) -> FontRenderMethod {
        FontRenderMethod::all()
    }
}

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

enum ImageType {
    Rgba(image::RgbaImage),
    Gpu(WgpuImageHandle),
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

fn load_path(path: &std::path::Path) -> Option<image::RgbaImage> {
    image::open(path).ok().map(|i| i.into_rgba8())
}

fn load_bytes(bytes: &[u8]) -> Option<image::RgbaImage> {
    image::load_from_memory(&bytes).ok().map(|i| i.into_rgba8())
}

fn load_custom(width: u32, height: u32, bpp: u32, bytes: &[u8]) -> Option<image::RgbaImage> {
    let dynamic = match bpp {
        1 => image::DynamicImage::ImageLuma8(image::GrayImage::from_pixel(
            width,
            height,
            *image::Luma::from_slice(bytes),
        )),
        2 => image::DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_pixel(
            width,
            height,
            *image::LumaA::from_slice(bytes),
        )),
        3 => image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            width,
            height,
            *image::Rgb::from_slice(bytes),
        )),
        4 => image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            width,
            height,
            *image::Rgba::from_slice(bytes),
        )),
        _ => return None,
    };

    Some(dynamic.to_rgba8())
}

fn create_image(
    id: Id,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bytes: &[u8],
) -> Option<WgpuImageHandle> {
    let image = image::load_from_memory(&bytes).ok()?;

    let wgpu_image = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("connui_wgpu::WgpuRenderer::image::{:?}", id)),
        size: wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    write_image(queue, &wgpu_image, &bytes, 4);

    Some(WgpuImageHandle::from(wgpu_image))
}

fn write_image(queue: &wgpu::Queue, texture: &wgpu::Texture, bytes: &[u8], bytes_per_pixel: u32) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_pixel * texture.width()),
            rows_per_image: Some(texture.height()),
        },
        wgpu::Extent3d {
            width: texture.width(),
            height: texture.height(),
            depth_or_array_layers: 1,
        },
    );
}
