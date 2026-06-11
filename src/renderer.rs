use crate::types::Color;

pub enum RenderCommand {
    DrawRect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    },
}
