struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@location(0) position: vec2<f32>,@location(1) tex_coords: vec2<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>( position, 0.0, 1.0);
    out.tex_coords = tex_coords;
    return out;
}

@group(0)
@binding(0)
var depth_texture: texture_depth_2d;
@group(0)
@binding(1)
var depth_sampler: sampler_comparison;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let depth = textureSampleCompare(depth_texture, depth_sampler, in.tex_coords, 0.5);
    return vec4<f32>(vec3<f32>(depth), 1.0);
}