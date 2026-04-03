use crate::{
    camera::Camera,
    entity::Entity,
    renderer::{EntityType, VertexIndicie},
    resources, world_units, Color, Mode, Point2D, Z,
};
use cgmath::{Point3, Vector3};
use std::collections::{HashMap, HashSet};

pub struct EngineContext<'a> {
    pub(crate) entities: &'a mut HashMap<u32, Entity>,
    pub(crate) entity_ids: &'a mut HashSet<u32>,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) camera: &'a mut Camera,
    pub(crate) background: &'a mut Color,
    pub(crate) mode: &'a mut Mode,
    pub(crate) text: &'a mut Vec<(String, f32, f32, u8)>,
    pub(crate) fps: &'a mut u32,
    pub(crate) dt: f32,
}

impl<'a> EngineContext<'a> {
    pub fn clear_background(&mut self, color: Color) {
        *self.background = color;
    }

    pub fn draw_circle(&mut self, id: u32, position: &Point2D, color: Color) -> bool {
        self.set_location(
            id,
            Vector3 {
                x: position.x,
                y: position.y,
                z: Z,
            },
        );
        self.set_color(id, color);
        self.entity_ids.insert(id)
    }

    pub fn add_circle(&mut self, id: u32, radius: f32) {
        let segments = 32; // increase for smoother circle
        let radius = world_units::meters_to_world(radius);

        let mut vertices = vec![];
        let mut indices = vec![];

        // center vertex
        vertices.push(crate::model::ModelVertex {
            position: [0.0, 0.0, Z],
            tex_coords: [0.5, 0.5],
            normal: [0.0, 0.0, 0.0],
        });

        // outer ring
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            vertices.push(crate::model::ModelVertex {
                position: [x, y, Z],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            });
        }

        // indices (triangle fan)
        for i in 1..=segments {
            indices.push(0);
            indices.push(i as u16);
            indices.push((i + 1) as u16);
        }

        let data = VertexIndicie {
            vertexes: vertices,
            indicies: indices,
            entity_type: EntityType::Circle,
        };

        self.entities.insert(
            id,
            Entity::new(
                id,
                data,
                1,
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: Z,
                },
                self.device,
            ),
        );
    }

    pub fn set_mode(&mut self, mode: Mode) {
        *self.mode = mode;
    }

    pub fn draw_entity(&mut self, id: u32, location: Vector3<f32>) {
        self.set_location(id, location);
        self.entity_ids.insert(id);
    }

    pub fn add_entity(&mut self, id: u32) -> bool {
        self.entity_ids.insert(id)
    }

    pub fn set_camera_eye(&mut self, eye: Point3<f32>) {
        self.camera.eye = eye;
    }

    pub fn set_camera_target(&mut self, target: Point3<f32>) {
        self.camera.target = target;
    }

    pub fn get_camera_eye(&self) -> Point3<f32> {
        self.camera.eye
    }

    pub fn get_camera_target(&self) -> Point3<f32> {
        self.camera.target
    }

    pub fn draw_rectangle(&mut self, id: u32, location: &Point2D, color: Color) -> bool {
        self.set_location(
            id,
            Vector3 {
                x: location.x,
                y: location.y,
                z: Z,
            },
        );
        self.set_color(id, color);
        self.entity_ids.insert(id)
    }

    pub fn add_rectangle(&mut self, id: u32, width: f32, height: f32) {
        let width = world_units::meters_to_world(width);
        let height = world_units::meters_to_world(height);

        let top_left = crate::model::ModelVertex {
            position: [0.0, 0.0, Z],
            tex_coords: [0.0, 1.0],
            normal: [0.0, 0.0, 0.0],
        };

        let top_right = crate::model::ModelVertex {
            position: [width, 0.0, Z],
            tex_coords: [1.0, 1.0],
            normal: [0.0, 0.0, 0.0],
        };

        let bottom_left = crate::model::ModelVertex {
            position: [0.0, 0.0 - height, Z],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };

        let bottom_right = crate::model::ModelVertex {
            position: [width, 0.0 - height, Z],
            tex_coords: [1.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };

        let entity_vertex_data = VertexIndicie {
            vertexes: vec![top_left, top_right, bottom_left, bottom_right],
            indicies: vec![0, 2, 1, 2, 3, 1],
            entity_type: EntityType::Rectangle,
        };

        self.entities.insert(
            id,
            Entity::new(
                id,
                entity_vertex_data,
                1,
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: Z,
                },
                self.device,
            ),
        );
    }

    pub fn add_entity_from_model(&mut self, id: u32, model_path: &str) -> anyhow::Result<()> {
        let model = resources::load_model(
            model_path,
            self.device,
            self.queue,
            self.texture_bind_group_layout,
        )?;
        let entity = Entity::from_model(
            id,
            model,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: Z,
            },
            self.device,
        );

        self.entities.insert(id, entity);
        Ok(())
    }

    // draws text, relative to the camera position, (0.0, 0.0) is top right
    pub fn draw_text(&mut self, location: Point2D, text: &str, font_size: u8) {
        self.text
            .push((String::from(text), location.x, location.y, font_size));
    }

    pub fn get_location(&mut self, id: u32) -> Option<Vector3<f32>> {
        if let Some(entity) = self.entities.get(&id) {
            Some(entity.location.into())
        } else {
            None
        }
    }

    pub fn set_location(&mut self, id: u32, location: Vector3<f32>) -> bool {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.location = [location.x, location.y, location.z];
            entity.rebuild_instance_buffer(self.device);
            true
        } else {
            false
        }
    }

    pub fn set_color(&mut self, id: u32, color: Color) -> bool {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.color = color.to_rgba();
            entity.rebuild_instance_buffer(self.device);
            true
        } else {
            false
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        *self.fps = fps;
    }
}
