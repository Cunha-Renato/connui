struct Instance {
    @location(0) position: vec2<f32>,      // top-left, in pixels
    @location(1) size: vec2<f32>,          // width/height, in pixels
    @location(2) uv: vec4<f32>,            // x, y, width, height
    @location(3) color: vec4<f32>,
    @location(4) corner_radius: vec4<f32>, // top-left, top-right, bottom-left, bottom-right
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) position: vec2<f32>, // top-left, in pixels
    @location(2) @interpolate(flat) size: vec2<f32>,     // width/height, in pixels
    @location(3) color: vec4<f32>,
    @location(4) @interpolate(flat) corner_radius: vec4<f32>,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32, instance: Instance) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
    );
    let corner = corners[vertex_index];

    // Snap both edges to whole pixels so straight sides are crisp
    let top_left = round(instance.position);
    let bottom_right = round(instance.position + instance.size);
    let size = bottom_right - top_left;

    // Pixels from the snapped top-left, expanded by PAD on every side
    let local = corner * size;
    let pixel_pos = top_left + local;

    let ndc_x = (pixel_pos.x / screen_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_pos.y / screen_size.y) * 2.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = instance.uv.xy + (local / size) * instance.uv.zw;
    out.position = top_left;
    out.size = size;
    out.color = instance.color;
    out.corner_radius = instance.corner_radius;
    return out;
}

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample first, before any branching (needs uniform control flow)
    let color = textureSample(tex, samp, in.uv) * in.color;

    // Pixel position relative to the rect's center
    let half = in.size * 0.5;
    let p = in.clip_position.xy - in.position - half;

    // Radius for this quadrant (x=tl, y=tr, z=bl, w=br)
    let left = p.x < 0.0;
    let r = select(
        select(in.corner_radius.w, in.corner_radius.z, left), // bottom
        select(in.corner_radius.y, in.corner_radius.x, left), // top
        p.y < 0.0,
    );

    // Offset from the corner circle's center, folded into the positive quadrant
    let q = abs(p) - (half - r);

    // Signed distance to the edge (negative inside).
    // length() is only evaluated inside the corner's r x r square.
    var d: f32;
    if q.x > 0.0 && q.y > 0.0 {
        d = length(q) - r;
    } else {
        d = max(q.x, q.y) - r;
    }

    let coverage = 1.0 - smoothstep(-0.5, 0.5, d);
    return vec4<f32>(color.rgb, color.a * coverage);
}