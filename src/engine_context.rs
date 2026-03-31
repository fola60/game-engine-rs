use std::collections::{HashMap, HashSet};
use crate::{
    camera::Camera,
    renderer::{EntityType, VertexIndicie},
    Color, Mode, Vertex, Point2D,
    entity::Entity
};
use cgmath::{Point3, Vector3};




pub struct EngineContext<'a> {
    pub(crate) entities: &'a mut HashMap<u32, Entity>,
    pub(crate) entity_ids: &'a mut HashSet<u32>,
    pub(crate) camera: &'a mut Camera,
    pub(crate) background: &'a mut Color,
    pub(crate) mode: &'a mut Mode,
    pub(crate) text: &'a mut Vec<(String, f32, f32, u8)>,
    pub(crate) fps: &'a mut u32,
    pub(crate) dt: f32
}

impl<'a> EngineContext<'a> {
    pub fn clear_background(&mut self, color: Color) {
        *self.background = color;      
    }

    pub fn draw_circle(&mut self, id: u32, position: Point2D, color: Color) -> bool {
        self.set_location(id, Vector3 { x: position.x, y: position.y, z: 0.0 });
        self.set_color(id, color);
        self.entity_ids.insert(id)
    }
    
    pub fn add_circle(&mut self, id: u32, radius: f32) {

        let segments = 32; // increase for smoother circle

        let mut vertices = vec![];
        let mut indices = vec![];

        // center vertex
        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            tex_coords: [0.5, 0.5],
        });

        // outer ring
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();

            vertices.push(Vertex {
                position: [x, y, 0.0],
                tex_coords: [0.0, 0.0],
            });
        }

        // indices (triangle fan)
        for i in 1..=segments {
            indices.push(i as u16);
            indices.push(0);
            indices.push((i + 1) as u16);
        }

        let data = VertexIndicie {
            vertexes: vertices,
            indicies: indices,
            entity_type: EntityType::Circle,
        };


        self.entities.insert(id, Entity::new(id, data, 1, Vector3 {x: 0.0, y: 0.0, z: 0.0}));
    }



    pub fn set_mode(&mut self, mode: Mode) {
        *self.mode = mode;
    }

    pub fn draw_entity(&mut self, id: u32, entity: Entity) {
        self.entities.insert(id, entity);
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

    pub fn draw_rectangle(&mut self, id: u32, location: Point2D, color: Color) -> bool {
        self.set_location(id, Vector3 { x: location.x, y: location.y, z: 0.0 });
        self.set_color(id, color);
        self.entity_ids.insert(id)
    }

    pub fn add_rectangle(&mut self, id: u32, width: f32, height: f32) {
        let z = 0.0;
        let top_left = Vertex { 
            position: [0.0, 0.0, z], 
            tex_coords: [0.0, 1.0]
        };

        let top_right = Vertex { 
            position: [width, 0.0, z], 
            tex_coords: [1.0, 1.0]
        };

        let bottom_left = Vertex { 
            position: [0.0, 0.0 - height, z], 
            tex_coords: [0.0, 0.0]
        };

        let bottom_right = Vertex { 
            position: [width, 0.0 - height, z], 
            tex_coords: [1.0, 0.0]
        };

        let entity_vertex_data = VertexIndicie { 
            vertexes: vec![top_left, top_right, bottom_left, bottom_right], indicies: vec![0, 1, 2, 2, 1, 3], entity_type: EntityType::Rectangle
        };

        
        self.entities.insert(id, Entity::new(id, entity_vertex_data, 1, Vector3 {x: 0.0, y: 0.0, z: 0.0}));
    }

    // draws text, relative to the camera position, (0.0, 0.0) is top right
    pub fn draw_text(&mut self, location: Point2D, text: &str, font_size: u8) {
        self.text.push((String::from(text), location.x, location.y, font_size));
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
            true
        } else {
            false
        }
    }

    pub fn set_color(&mut self, id: u32, color: Color) -> bool {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.color = color.to_rgba();
            true
        } else {
            false
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        *self.fps = fps;
    }
}
