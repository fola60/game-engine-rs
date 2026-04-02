use crate::{model::ModelVertex, Point2D};

pub enum EntityType {
    Rectangle,
    Circle,
    VertIndicie,
    Model,
}

pub(crate) struct VertexIndicie {
    pub(crate) vertexes: Vec<ModelVertex>,
    pub(crate) indicies: Vec<u16>,
    pub(crate) entity_type: EntityType,
}

pub struct Renderer {
    pub(crate) entity_vertex_data: Vec<VertexIndicie>,
    pub(crate) screen_width: u32,
    pub(crate) screen_height: u32,
}

impl Renderer {
    pub fn draw_circle(&self, _location: Point2D, _radius: u32) {}

    pub fn draw_rectangle(&mut self, location: Point2D, width: u32, height: u32) {
        let top_right = ModelVertex {
            position: [location.x + 1.0 / width as f32, location.y, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };
        let top_left = ModelVertex {
            position: [location.x, location.y, 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };
        let bottom_left = ModelVertex {
            position: [location.x, location.y - (1.0 / height as f32), 0.0],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };
        let bottom_right = ModelVertex {
            position: [
                location.x + 1.0 / width as f32,
                location.y - (1.0 / height as f32),
                0.0,
            ],
            tex_coords: [0.0, 0.0],
            normal: [0.0, 0.0, 0.0],
        };
        let entity_vertex_data = VertexIndicie {
            vertexes: vec![top_left, top_right, bottom_left, bottom_right],
            indicies: vec![0, 1, 2, 2, 3, 1],
            entity_type: EntityType::Rectangle,
        };
        self.entity_vertex_data.push(entity_vertex_data);
    }
}
