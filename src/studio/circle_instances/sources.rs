use bytemuck::{Pod, Zeroable};

use crate::painter::Sandy;

const MAX_CIRCLES: usize = 100;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct Uniforms {
    time: f32,
}

struct Circle {
    position: [f32; 2],
    radius: f32,
    color: [f32; 4],
}

fn gen_circles() {
    let mut circles = Vec::with_capacity(MAX_CIRCLES);
    for _ in 0..MAX_CIRCLES {
        let circle = Circle {
            position: [0.0, 0.0],
            radius: 0.0,
            color: [0.0, 0.0, 0.0, 0.0],
        };
        circles.push(circle);
    }
}

// 简述：： 顶点缓冲区去包括所有的圆的位置和半径，而颜色是通过 uniform 传递给着色器的。
// 或者 实例化一个圆的状态， 用实例数据记录大小和新位置
// 
