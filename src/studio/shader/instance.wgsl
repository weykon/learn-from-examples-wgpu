struct Uniforms { 
    @location(0) dt:f32,
    @location(1) time:f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

struct InstanceInput {
    @location(2) position: vec3f,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
     var out: VertexOutput;
    out.color = model.color;

    // 计算圆周运动的位置
    var angle = uniforms.time + instance.position.x; // 使用时间和实例位置计算角度
    var radius = 0.8; // 圆的半径
    var animated_position = vec3f(
        model.position.x + radius * cos(angle),
        model.position.y + radius * sin(angle),
        model.position.z
    );

    out.clip_position = vec4f(animated_position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let animated_color = vec3f(
        abs(sin( in.color.r * uniforms.time * 0.5)),
        abs(sin( in.color.g * uniforms.time * 0.4)),
        abs(sin( in.color.b * uniforms.time * 0.3))
    );
    return vec4f(animated_color, 1.0);
}