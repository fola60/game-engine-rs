use cgmath::Point3;
use winit::keyboard::KeyCode;

use crate::Point2D;

pub(crate) struct CameraController {
    pub target: Point2D,
    pub eye: Point2D,
    pub offset: Point2D,
    pub rotation: f32
}





impl CameraController {
    pub(crate) fn new(eye: Point2D) -> Self {
        Self {
            target: Point2D { x: 0.0, y: 0.0 },
            eye: eye,
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        true
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera, width: u32, height: u32) {
        camera.eye = Point3 {x: self.eye.x + self.offset.x, y: self.eye.y + self.offset.y, z: 0.0};
        camera.target = Point3 {x: self.target.x, y: self.target.y, z: -1.0};
        camera.rotation = self.rotation;
        camera.aspect = (width as f32 / height as f32);
    }

    
    pub fn set_camera_eye(&mut self, location: Point2D) {
        self.eye = location;
    }

    pub fn set_camera_target(&mut self, location: Point2D) {
        self.target = location
    }

    pub fn set_camera_rotation(&mut self, angle: f32) {
        self.rotation = angle;
    }

     pub fn get_camera_eye(&self) -> Point2D{
        self.eye.clone()
    }

    pub fn get_camera_target(&mut self) -> Point2D {
        self.target.clone()
    }

    pub fn get_camera_rotation(&mut self) -> f32{
        self.rotation
    }
    
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub(crate) view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub(crate) fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub(crate) fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub struct Camera {
    pub(crate) eye: cgmath::Point3<f32>,
    pub(crate) target: cgmath::Point3<f32>,
    pub(crate) up: cgmath::Vector3<f32>,
    pub(crate) aspect: f32,
    pub(crate) fovy: f32,
    pub(crate) znear: f32,
    pub(crate) zfar: f32,
    pub(crate) rotation: f32
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::from_cols(
    cgmath::Vector4::new(1.0, 0.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 1.0, 0.0, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 0.0),
    cgmath::Vector4::new(0.0, 0.0, 0.5, 1.0),
);

impl Camera {
    pub(crate) fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        // 3.
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}
