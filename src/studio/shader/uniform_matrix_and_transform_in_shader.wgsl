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

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    var model_position = vec4f(model.position, 1.0);
    let angle = time_uniforms.time;
    model_position.x *= sin(angle);
    model_position.y *= cos(angle);
    let scale = sin(time_uniforms.time) * 0.5 + 1.5;
    model_position.y *= scale;
    model_position.x *= scale;
    out.clip_position = env_uniforms.proj * env_uniforms.view * env_uniforms.world * model_position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var r = abs(sin(time_uniforms.time*2. + in.clip_position.y));
    return vec4f( r, in.color.g, in.color.b , 1.0);
}