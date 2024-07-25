struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
};

struct VertexOutput { 
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main( vi: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vi.pos.xy, 0.0, 1.0);
    out.tex_coords = vi.tex_coords;
    return out;
}

@group(0)
@binding(0)
var tex: texture_2d<f32>;
@group(0)
@binding(1)
var sam: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(tex, sam, vertex.tex_coords, 0.0);
}
