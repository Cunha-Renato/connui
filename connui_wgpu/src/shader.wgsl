struct Instance {
    @location(0) position: vec2<f32>, // top-left, in pixels
    @location(1) size: vec2<f32>,     // width/height, in pixels
    @location(2) uv: vec4<f32>,       // x, y, width, height — matches Instance::new's [uv.x, uv.y, uv.width, uv.height]
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: Instance) -> VertexOutput {
    // Two triangles, 6 vertices — matches `pass.draw(0..6, draw.start..draw.start + draw.len)`.
    // Winding doesn't matter (cull_mode: None), so this is just whichever
    // split avoids a diagonal seam artifact.
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vertex_index];

    let pixel_pos = instance.position + corner * instance.size;
    let ndc_x = (pixel_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / screen_size.y) * 2.0; // pixel y grows downward, NDC y grows upward

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = instance.uv.xy + corner * instance.uv.zw;
    out.color = instance.color;
    return out;
}

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Solid-color quads sample the 1x1 white texture (uv defaults to
    // [0,0,1,1] in Instance::new when no uv rect is given), so this one
    // multiply covers both textured and solid draws without a branch.
    return textureSample(tex, samp, in.uv) * in.color;
}
