struct Uniforms { 
    @location(0) delta_time:f32,
    @location(1) time:f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

struct InstanceInput {
    @location(1) radius: f32,
    @location(2) position: vec2f,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let speed = 9.8 / instance.radius;
    var world_pos = (model.position * instance.radius) + instance.position;
 
    var angle = instance.position + uniforms.time;
    angle = angle * speed * 0.01;

    // 定义屏幕中心
    var center = vec2f(0.0, 0.0);

    // 将 world_pos 平移到屏幕中心
    world_pos -= center;

    // 进行旋转变换
    world_pos.x = world_pos.x * cos(angle.x) - world_pos.y * sin(angle.x);
    world_pos.y = world_pos.y * sin(angle.y) + world_pos.y * cos(angle.y);

    // 将 world_pos 平移回原来的位置
    world_pos += center;

    world_pos.x += sin(angle.x) * 0.1;
    world_pos.y += cos(angle.y) * 0.1;


    out.clip_position = vec4f(world_pos, 1., 1.0);
    out.color = vec3f(
        sin( uniforms.time * speed * 0.01 ) * 0.5 + 0.5,
        cos( uniforms.time * speed * 0.01 ) * 0.5 + 0.5,
        0.3
    );
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}