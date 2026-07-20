use connui::{renderer::RendererImageHandle, types::Id};
use image::Pixel;
use std::sync::Arc;

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

pub(crate) struct WgpuImageHandleInner {
    pub handle: WgpuImageHandle,
    pub bind_group: wgpu::BindGroup,
}

pub(crate) enum ImageType {
    Rgba(image::RgbaImage),
    Gpu(WgpuImageHandle),
}

pub(crate) fn load_path(path: &std::path::Path) -> Option<image::RgbaImage> {
    image::open(path).ok().map(|i| i.into_rgba8())
}

pub(crate) fn load_bytes(bytes: &[u8]) -> Option<image::RgbaImage> {
    image::load_from_memory(&bytes).ok().map(|i| i.into_rgba8())
}

pub(crate) fn load_custom(
    width: u32,
    height: u32,
    bpp: u32,
    bytes: &[u8],
) -> Option<image::RgbaImage> {
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

pub(crate) fn create_image(
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

pub(crate) fn write_image(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    bytes: &[u8],
    bytes_per_pixel: u32,
) {
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
