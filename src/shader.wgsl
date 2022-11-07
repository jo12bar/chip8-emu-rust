struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;

    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);

    return out;
}

@group(0) @binding(0)
var t_display: texture_2d<f32>;
@group(0) @binding(1)
var s_display: sampler;

struct ScreenSizeUniform {
    size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> screen_size: ScreenSizeUniform;

fn aspect_2d_scale_2d(size: vec2<f32>, scale: vec2<f32>) -> vec2<f32> {
    return scale / size;
}

fn aspect_2d(size: vec2<f32>, scale: f32) -> vec2<f32> {
    return aspect_2d_scale_2d(size, vec2(scale));
}

fn aspect_contain_2d(size: vec2<f32>) -> vec2<f32> {
    return aspect_2d(size, max(size.x, size.y));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Scale texture coordinates from [0.0, 1.0] to [-1.0, 1.0]
    var st = 2.0 * (in.tex_coords - 0.5);

    // Scale to fit the aspect ratio of the screen, similar to CSS's background-size: contain
    let texture_size: vec2<i32> = textureDimensions(t_display);
    let size_ratio = vec2<f32>(texture_size) / screen_size.size;
    st = st * aspect_contain_2d(size_ratio);

    // Re-scale texture coordinates from [-1.0, 1.0] to [0.0, 1.0]
    st = (0.5 * st) + 0.5;

    // Sample the texture
    let pixel = textureSample(t_display, s_display, st);

    // Make anything outside the texture coordinate range of [0.0, 1.0] black
    let limit = abs(floor(st));
    let black = vec4(0.0, 0.0, 0.0, 1.0);

    return mix(black, pixel, step(max(limit.x, limit.y), 0.0));
}
