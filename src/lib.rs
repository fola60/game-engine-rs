use cgmath::prelude::*;

pub mod app_window;
pub mod engine;
pub mod state;
pub mod texture;
pub mod camera;
pub mod renderer;
pub mod entity;
pub mod engine_context;
// Draw a 2d circle 


pub struct Point2D {
    pub x: f32,
    pub y: f32
}

impl Clone for Point2D {
    fn clone(&self) -> Self {
        Point2D { x: self.x, y: self.y }
    }
}

impl Default for Point2D {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

pub enum Color {
    Red,
    Green,
    Blue,
    White,
    Black,
    Yellow,
    Cyan,
    Magenta,
    Custom(f32, f32, f32, f32),
}

impl Color {
    pub fn to_rgba(&self) -> [f32; 4] {
        match self {
            Color::Red     => [1.0, 0.0, 0.0, 1.0],
            Color::Green   => [0.0, 1.0, 0.0, 1.0],
            Color::Blue    => [0.0, 0.0, 1.0, 1.0],
            Color::White   => [1.0, 1.0, 1.0, 1.0],
            Color::Black   => [0.0, 0.0, 0.0, 1.0],
            Color::Yellow  => [1.0, 1.0, 0.0, 1.0],
            Color::Cyan    => [0.0, 1.0, 1.0, 1.0],
            Color::Magenta => [1.0, 0.0, 1.0, 1.0],
            Color::Custom(r, g, b, a) => [*r, *g, *b, *a],
        }
    }
}


// lib.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2]
}




impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, 
                }
            ]
        }
    }
}



pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, -1.0], tex_coords: [0.4131759, 0.00759614], }, // A
    Vertex { position: [-0.49513406, 0.06958647, -1.0], tex_coords: [0.0048659444, 0.43041354], }, // B
    Vertex { position: [-0.21918549, -0.44939706, -1.0], tex_coords: [0.28081453, 0.949397], }, // C
    Vertex { position: [0.35966998, -0.3473291, -1.0], tex_coords: [0.85967, 0.84732914], }, // D
    Vertex { position: [0.44147372, 0.2347359, -1.0], tex_coords: [0.9414737, 0.2652641], }, // E
];


pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

struct Instance {
    position: cgmath::Vector3<f32>,
    rotation: cgmath::Quaternion<f32>,
    vertex_offset: cgmath::Vector3<f32>,
    color: cgmath::Vector4<f32>
}

impl Instance {
    fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
            color: self.color.into(),
            vertex_offset: self.vertex_offset.into(),
            _padding: 0.0
        }
    }
}

// NEW!
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    color: [f32; 4],
    vertex_offset: [f32; 3],
    _padding: f32
}

impl InstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 20]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ],
        }
    }
}

struct Size {
    width: u32,
    height: u32
}

pub enum Mode {
    MODE2D,
    Mode3D
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
