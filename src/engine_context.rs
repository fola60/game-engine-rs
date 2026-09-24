use crate::{
    Color, Dimension, Gesture, Point2D, ThreeD, Transform, TwoD, Z, camera::Camera,
    render_object::RenderObject, resources, state::State,
};
use cgmath::{Point3, Quaternion, Vector3};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

pub struct EngineContext<'a, D: Dimension> {
    pub scene: &'a mut crate::scene::Scene<D>,
    pub(crate) entities: &'a mut HashMap<String, RenderObject>,
    pub(crate) entity_ids: &'a mut HashSet<String>,
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) texture_bind_group_layout: &'a wgpu::BindGroupLayout,
    pub(crate) camera: &'a mut Camera,
    pub(crate) background: &'a mut Color,
    pub(crate) text: &'a mut Vec<(String, f32, f32, u8)>,
    pub(crate) gesture: &'a mut Option<Gesture>,
    pub(crate) fps: &'a mut u32,
    dimension: PhantomData<D>,
}

impl<'a, D: Dimension> EngineContext<'a, D> {
    pub(crate) fn new(
        state: &'a mut State,
        fps: &'a mut u32,
        scene: &'a mut crate::scene::Scene<D>,
    ) -> Self {
        Self {
            scene,
            entities: &mut state.entities,
            entity_ids: &mut state.entity_ids,
            device: &state.device,
            queue: &state.queue,
            texture_bind_group_layout: &state.texture_bind_group_layout,
            camera: &mut state.camera,
            background: &mut state.background,
            text: &mut state.text,
            gesture: &mut state.gesture,
            fps,
            dimension: PhantomData,
        }
    }

    /// Register an entity for automatic lifecycle callbacks and drawing.
    /// IDs in the scene are independent of the legacy immediate drawing API.
    pub fn spawn<E: crate::Entity<D>>(&mut self, id: impl Into<String>, entity: E) -> bool {
        self.scene.spawn(id, entity)
    }

    pub fn despawn(&mut self, id: &str) -> bool {
        self.scene.despawn(id)
    }

    pub fn entity_mut<E: crate::Entity<D>>(&mut self, id: &str) -> Option<&mut E> {
        self.scene.get_mut(id)
    }

    pub fn get_gesture(&self) -> Option<Gesture> {
        *self.gesture
    }

    pub fn take_gesture(&mut self) -> Option<Gesture> {
        self.gesture.take()
    }

    pub fn clear_background(&mut self, color: Color) {
        *self.background = color;
    }

    pub fn draw_circle(&mut self, id: &str, position: &Point2D, color: Color) -> bool {
        self.set_location(
            id,
            Vector3 {
                x: position.x,
                y: position.y,
                z: Z,
            },
        );
        self.set_color(id, color);
        self.entity_ids.insert(id.to_owned())
    }

    pub fn draw_cube(&mut self, id: &str, position: Vector3<f32>, color: Color) -> bool {
        self.set_location(id, position);
        self.set_color(id, color);
        self.entity_ids.insert(id.to_owned())
    }

    pub fn add_circle(&mut self, id: &str, radius: f32) {
        let mesh = crate::Mesh::<TwoD>::circle(radius);
        self.entities.insert(
            id.to_owned(),
            RenderObject::new(
                id.to_owned(),
                (*mesh.data).clone(),
                1,
                Vector3::new(0.0, 0.0, Z),
                self.device,
            ),
        );
    }

    pub fn draw_entity(&mut self, id: &str, location: Vector3<f32>) {
        self.set_location(id, location);
        self.entity_ids.insert(id.to_owned());
    }

    pub fn add_entity(&mut self, id: &str) -> bool {
        self.entity_ids.insert(id.to_owned())
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

    pub fn draw_rectangle(&mut self, id: &str, location: &Point2D, color: Color) -> bool {
        self.set_location(
            id,
            Vector3 {
                x: location.x,
                y: location.y,
                z: Z,
            },
        );
        self.set_color(id, color);
        self.entity_ids.insert(id.to_owned())
    }

    pub fn add_rectangle(&mut self, id: &str, width: f32, height: f32) {
        let mesh = crate::Mesh::<TwoD>::rectangle(width, height);
        self.entities.insert(
            id.to_owned(),
            RenderObject::new(
                id.to_owned(),
                (*mesh.data).clone(),
                1,
                Vector3::new(0.0, 0.0, Z),
                self.device,
            ),
        );
    }

    pub fn add_cube(&mut self, id: &str, width: f32, height: f32, length: f32) {
        let mesh = crate::Mesh::<ThreeD>::cube(width, height, length);
        self.entities.insert(
            id.to_owned(),
            RenderObject::new(
                id.to_owned(),
                (*mesh.data).clone(),
                1,
                Vector3::new(0.0, 0.0, Z),
                self.device,
            ),
        );
    }

    pub fn add_entity_from_model(&mut self, id: &str, model_path: &str) -> anyhow::Result<()> {
        let model = resources::load_model(
            model_path,
            self.device,
            self.queue,
            self.texture_bind_group_layout,
        )?;
        let entity = RenderObject::from_model(
            id.to_owned(),
            model,
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: Z,
            },
            self.device,
        );

        self.entities.insert(id.to_owned(), entity);
        Ok(())
    }

    // draws text, relative to the camera position, (0.0, 0.0) is top right
    pub fn draw_text(&mut self, location: Point2D, text: &str, font_size: u8) {
        self.text
            .push((String::from(text), location.x, location.y, font_size));
    }

    pub fn get_transform(&self, id: &str) -> Option<Transform> {
        self.entities.get(id).map(|entity| entity.transform)
    }

    pub fn set_transform(&mut self, id: &str, transform: Transform) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.transform = transform;
            entity.instance_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn get_location(&self, id: &str) -> Option<Vector3<f32>> {
        self.entities
            .get(id)
            .map(|entity| entity.transform.position)
    }

    pub fn set_location(&mut self, id: &str, location: Vector3<f32>) -> bool {
        self.set_position(id, location)
    }

    pub fn set_position(&mut self, id: &str, position: Vector3<f32>) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.transform.position = position;
            entity.instance_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_rotation(&mut self, id: &str, rotation: Quaternion<f32>) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.transform.rotation = rotation;
            entity.instance_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_scale(&mut self, id: &str, scale: Vector3<f32>) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.transform.scale = scale;
            entity.instance_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_color(&mut self, id: &str, color: Color) -> bool {
        if let Some(entity) = self.entities.get_mut(id) {
            entity.color = color.to_rgba();
            entity.instance_dirty = true;
            true
        } else {
            false
        }
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        *self.fps = fps.max(1);
    }
}

impl EngineContext<'_, TwoD> {
    pub fn draw(&mut self, id: &str, location: &Point2D, color: Color) -> bool {
        self.set_location(id, Vector3::new(location.x, location.y, Z));
        self.set_color(id, color);
        self.entity_ids.insert(id.to_owned())
    }
}

impl EngineContext<'_, ThreeD> {
    pub fn draw(&mut self, id: &str, location: &Vector3<f32>, color: Color) -> bool {
        self.set_location(id, *location);
        self.set_color(id, color);
        self.entity_ids.insert(id.to_owned())
    }
}
