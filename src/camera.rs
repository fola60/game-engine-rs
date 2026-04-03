use cgmath::Point3;
use winit::keyboard::KeyCode;

use crate::{world_units, Point2D};

pub(crate) struct CameraController {
    pub target: Point2D,
    pub eye: Point2D,
    pub offset: Point2D,
    pub rotation: f32,
}

impl CameraController {
    pub(crate) fn new(eye: Point2D) -> Self {
        Self {
            target: Point2D { x: 0.0, y: 0.0 },
            eye: eye,
            offset: Point2D { x: 0.0, y: 0.0 },
            rotation: 0.0,
        }
    }

    pub(crate) fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        true
    }

    pub(crate) fn update_camera(&self, camera: &mut Camera, width: u32, height: u32) {
        camera.eye = Point3 {
            x: self.eye.x + self.offset.x,
            y: self.eye.y + self.offset.y,
            z: camera.eye.z,
        };
        camera.target = Point3 {
            x: self.target.x,
            y: self.target.y,
            z: camera.target.z,
        };
        camera.rotation = self.rotation;
        camera.aspect = width as f32 / height as f32;
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

    pub fn get_camera_eye(&self) -> Point2D {
        self.eye.clone()
    }

    pub fn get_camera_target(&mut self) -> Point2D {
        self.target.clone()
    }

    pub fn get_camera_rotation(&mut self) -> f32 {
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

pub enum Projection {
    Perspective {
        fovy: f32,
        znear: f32,
        zfar: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        znear: f32,
        zfar: f32,
    },
}

pub struct Camera {
    pub(crate) eye: cgmath::Point3<f32>,
    pub(crate) target: cgmath::Point3<f32>,
    pub(crate) up: cgmath::Vector3<f32>,
    pub(crate) aspect: f32,
    pub(crate) rotation: f32,
    pub(crate) projection: Projection,
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
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let proj = match self.projection {
            Projection::Perspective { fovy, znear, zfar } => {
                cgmath::perspective(cgmath::Deg(fovy), self.aspect, znear, zfar)
            }
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                znear,
                zfar,
            } => cgmath::ortho(left, right, bottom, top, znear, zfar),
        };

        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub(crate) fn get_2d_camera(width: f32, height: f32) -> Self {
        let aspect = width / height;
        let ortho_height =
            world_units::meters_to_world(world_units::ORTHOGRAPHIC_VIEW_HEIGHT_METERS);
        let ortho_width = ortho_height * aspect;

        Camera {
            eye: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            target: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            aspect,
            rotation: 0.0,
            projection: Projection::Orthographic {
                left: -ortho_width * 0.5,
                right: ortho_width * 0.5,
                bottom: -ortho_height * 0.5,
                top: ortho_height * 0.5,
                znear: world_units::meters_to_world(world_units::CAMERA_2D_ZNEAR_METERS),
                zfar: world_units::meters_to_world(world_units::CAMERA_2D_ZFAR_METERS),
            },
        }
    }

    pub(crate) fn get_3d_camera(width: f32, height: f32) -> Self {
        let camera_distance = world_units::meters_to_world(world_units::CAMERA_3D_DISTANCE_METERS);

        Camera {
            eye: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: camera_distance,
            },
            target: cgmath::Point3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            up: cgmath::Vector3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            aspect: width / height,
            rotation: 0.0,
            projection: Projection::Perspective {
                fovy: world_units::CAMERA_3D_FOVY_DEGREES,
                znear: world_units::meters_to_world(world_units::CAMERA_3D_ZNEAR_METERS),
                zfar: world_units::meters_to_world(world_units::CAMERA_3D_ZFAR_METERS),
            },
        }
    }
}
