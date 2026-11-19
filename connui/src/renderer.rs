use crate::prelude::*;
use crate::tree::LayoutElementRef;
use crate::{image, types::Color};

pub trait Renderer: Sized {
    type ImageHandle: RendererImageHandle;

    fn scale_factor(&self) -> f32;
    fn set_scale_factor(&mut self, scale_factor: f32);

    // Render Commands.
    fn draw_quad(
        &mut self,
        rect: &PRect,
        corner_radius: &PCorners,
        color: Color,
        uv: Option<&Rect<f32>>,
    );

    fn push_clip(&mut self, clip: &Clip<u32>);
    fn pop_clip(&mut self);

    fn push_image(&mut self, id: Id);
    fn pop_image(&mut self);

    fn load_image(&mut self, handle: image::Handle<Self>) -> Option<Self::ImageHandle>;
    fn write_image<'a>(
        &mut self,
        id: &Id,
        write_op: image::WriteOp<'a>,
        fallback: Option<image::Handle<Self>>,
    );
}

pub trait RendererImageHandle: Clone + Send + Sync {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

#[derive(Debug)]
pub struct RenderElement {
    pub rect: PRect,
    pub corner_radius: PCorners,
    pub clip: Clip<u32>,
}
impl RenderElement {
    pub(crate) fn new<R: Renderer>(layout_element: LayoutElementRef, renderer: &mut R) -> Self {
        let borrow = layout_element.borrow();
        let scale_factor = renderer.scale_factor();

        let rect = borrow.rect.map(|r| r.to_physical(scale_factor));
        let clip_rect_inner = borrow
            .clip
            .rect
            .map(|c| c.to_physical(scale_factor).inner());
        let clip_corners = borrow.clip.corners.map(|c| c.to_physical(scale_factor));

        let x0 = clip_rect_inner.x();
        let y0 = clip_rect_inner.y();
        let x1 = clip_rect_inner.x() + clip_rect_inner.width();
        let y1 = clip_rect_inner.y() + clip_rect_inner.height();

        let clip_x = x0.floor().max(0.0) as u32;
        let clip_y = y0.floor().max(0.0) as u32;
        let clip_w = (x1 - x0).ceil().max(0.0) as u32;
        let clip_h = (y1 - y0).ceil().max(0.0) as u32;

        let clip = Clip {
            rect: Rect::new(clip_x, clip_y, clip_w, clip_h),
            corners: clip_corners.map(|c| c.inner().max(0.0) as u32),
        };

        Self {
            rect,
            corner_radius: clip_corners,
            clip,
        }
    }
}
