use cgmath::prelude::*;

pub mod app_window;
pub mod engine;
pub mod state;
pub mod texture;
pub mod camera;
pub mod renderer;
pub mod entity;
pub mod engine_context;
pub mod model;
pub mod resources;
pub mod text;
pub mod gesture;
pub mod world_units;

// Draw a 2d circle 

pub const Z: f32 = 0.0;
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
    LightRed,
    Green,
    LightGreen,
    Blue,
    LightBlue,
    White,
    LightGray,
    Black,
    LightBlack,
    Yellow,
    LightYellow,
    Cyan,
    LightCyan,
    Magenta,
    LightMagenta,
    Custom(f32, f32, f32, f32),
}

#[derive(Clone, Copy, Debug)]
pub enum Gesture {
    SwipeUp(f32),
    SwipeDown(f32),
    SwipeLeft(f32),
    SwipeRight(f32)
}

impl Gesture {
    pub fn get_gesture(
        start: &Point2D,
        end: &Point2D,
        duration_secs: f32,
        min_distance: f32,
        max_duration_secs: f32,
    ) -> Option<Self> {
        if duration_secs > max_duration_secs {
            return None;
        }

        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < min_distance {
            return None;
        }

        let angle_deg = dy.atan2(dx).to_degrees();

        if (-30.0..=30.0).contains(&angle_deg) {
            Some(Gesture::SwipeRight(angle_deg))
        } else if (45.0..=135.0).contains(&angle_deg) {
            Some(Gesture::SwipeDown(angle_deg))
        } else if angle_deg >= 150.0 || angle_deg <= -150.0 {
            Some(Gesture::SwipeLeft(angle_deg))
        } else {
            Some(Gesture::SwipeUp(angle_deg))
        }
    }
}

impl Color {
    pub fn to_rgba(&self) -> [f32; 4] {
        match self {
            Color::Red     => [1.0, 0.0, 0.0, 1.0],
            Color::LightRed => [1.0, 0.5, 0.5, 1.0],
            Color::Green   => [0.0, 1.0, 0.0, 1.0],
            Color::LightGreen => [0.5, 1.0, 0.5, 1.0],
            Color::Blue    => [0.0, 0.0, 1.0, 1.0],
            Color::LightBlue => [0.5, 0.5, 1.0, 1.0],
            Color::White   => [1.0, 1.0, 1.0, 1.0],
            Color::LightGray => [0.85, 0.85, 0.85, 1.0],
            Color::Black   => [0.0, 0.0, 0.0, 1.0],
            Color::LightBlack => [0.2, 0.2, 0.2, 1.0],
            Color::Yellow  => [1.0, 1.0, 0.0, 1.0],
            Color::LightYellow => [1.0, 1.0, 0.6, 1.0],
            Color::Cyan    => [0.0, 1.0, 1.0, 1.0],
            Color::LightCyan => [0.6, 1.0, 1.0, 1.0],
            Color::Magenta => [1.0, 0.0, 1.0, 1.0],
            Color::LightMagenta => [1.0, 0.6, 1.0, 1.0],
            Color::Custom(r, g, b, a) => [*r, *g, *b, *a],
        }
    }
}


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
    Mode2D,
    Mode3D
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
