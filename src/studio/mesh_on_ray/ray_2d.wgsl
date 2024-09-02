struct TimeUniforms { 
    @location(0) dt:f32,
    @location(1) time:f32,
};
struct RayUniforms { 
    @location(0) origin:vec3<f32>,
    @location(1) direction:vec3<f32>,
    @location(2) intensity:f32,
};

@group(0) @binding(0) var<uniform> time_uniforms: Uniforms;
@group(1) @binding(0) var<uniform> ray_uniforms: RayUniforms;

struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
};

struct InstanceInput {
    @location(2) position: vec3f,
    @location(3) rotation: vec3f,
    @location(4) on_type: u32,
    @location(5) scale: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

// 同样的两个@location(0), 
// 这个location的意义是看当用在什么的结构体中才
// 代表着当前用处的0位置的数据

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
    // 正常显示从vertex的position和instance里面的scale和rotation
    out.clip_position = vec4f(rotated_position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    
    return vec4f(animated_color, 1.0);
}