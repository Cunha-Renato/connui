struct Clip {
    position: vec2<f32>, // top-left, in pixels
    size: vec2<f32>,     // width/height, in pixels
    corner_radius: vec4<f32>, // top-left, top-right, bottom-left, bottom-right
    parent: u32,
}

struct Instance {
    @location(0) position: vec2<f32>,      // top-left, in pixels
    @location(1) size: vec2<f32>,          // width/height, in pixels
    @location(2) uv: vec4<f32>,            // x, y, width, height
    @location(3) color: vec4<f32>,
    @location(4) corner_radius: vec4<f32>, // top-left, top-right, bottom-left, bottom-right
    @location(5) clip_idx: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) position: vec2<f32>, // top-left, in pixels
    @location(2) @interpolate(flat) size: vec2<f32>,     // width/height, in pixels
    @location(3) color: vec4<f32>,
    @location(4) @interpolate(flat) corner_radius: vec4<f32>,
    @location(5) @interpolate(flat) clip_idx: u32,
};

@group(0) @binding(0) var<uniform> screen_size: vec2<f32>;
@group(0) @binding(1) var<storage, read> clips: array<Clip>;

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
    out.clip_idx = instance.clip_idx;
    out.corner_radius = instance.corner_radius;
    return out;
}

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

// Signed distance to a rounded rect (negative inside).
// Radii order: x=tl, y=tr, z=bl, w=br.
fn rrect_distance(frag: vec2<f32>, pos: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let half_size = size * 0.5;
    let p = frag - pos - half_size;

    let left = p.x < 0.0;
    let r = select(
        select(radii.w, radii.z, left), // bottom
        select(radii.y, radii.x, left), // top
        p.y < 0.0,
    );

    let q = abs(p) - (half_size - r);
    if q.x > 0.0 && q.y > 0.0 {
        return length(q) - r;
    }
    return max(q.x, q.y) - r;
}

fn edge_coverage(d: f32) -> f32 {
    return 1.0 - smoothstep(-0.5, 0.5, d);
}

// Product of the coverage of every clip in the parent chain.
fn clip_coverage(clip_idx: u32, frag: vec2<f32>) -> f32 {
    var cov = 1.0;

    var i = clip_idx;
    loop {
        if cov == 0.0 {
            break;
        }

        let c = clips[i];

        cov *= edge_coverage(rrect_distance(frag, c.position, c.size, c.corner_radius));

        if i == 0 && c.parent == 0 {
            break;
        }

        i = c.parent;
    }

    return cov;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample first, before any branching (needs uniform control flow)
    let color = textureSample(tex, samp, in.uv) * in.color;
    let frag = in.clip_position.xy;

    var coverage = clip_coverage(in.clip_idx, frag);

    // The instance's own rounded edge
    if any(in.corner_radius > vec4<f32>(0.0)) {
        coverage *= edge_coverage(rrect_distance(frag, in.position, in.size, in.corner_radius));
    }

    return vec4<f32>(color.rgb, color.a * coverage);
}