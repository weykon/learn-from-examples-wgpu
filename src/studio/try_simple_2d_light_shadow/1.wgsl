struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vertex_position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> light_position: vec2<f32>;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;
    let light_position_4d = vec4<f32>(light_position, -0.1, 1.0);
    out.vertex_position = vec4<f32>(position, 1.0);
    out.clip_position = light_position_4d * out.vertex_position;
    return out;
}