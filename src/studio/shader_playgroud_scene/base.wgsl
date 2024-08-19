struct EnvUniforms { 
    world: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> env_uniforms: EnvUniforms;


struct TimeUniforms { 
    time: f32,
}
@group(0) @binding(1) var<uniform> time_uniforms: TimeUniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.color = vec4f(color, 1.0);

    // result.position = vec4f(position,1.0);
    result.position = env_uniforms.proj * env_uniforms.view * env_uniforms.world * vec4f(position,1.0);

    return result;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}