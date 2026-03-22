use wgpu::util::DeviceExt;
use cgmath::{InnerSpace, Rotation3, Zero};
use crate::{
    Instance, Point2D, Vertex, texture::Texture
};

pub struct Entity {
    pub (crate) vertex_buffer: wgpu::Buffer,
    pub (crate) index_buffer: wgpu::Buffer,
    pub (crate) instances: Vec<Instance>,
    pub (crate) instance_buffer: wgpu::Buffer,
    pub (crate) num_indices: u32,
    pub (crate) diffuse_bind_group: Option<wgpu::BindGroup>,
    pub (crate) diffuse_texture: Option<Texture>
}

impl Entity {
    pub (crate) fn new_rectangle(device: &mut wgpu::Device , vertex_data: &VertexIndicie) -> Entity {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data.vertexes),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let indicies: [u16; 6]= [
            0, 1, 2,
            2, 1, 3
        ];

        

        let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indicies),
            usage: wgpu::BufferUsages::INDEX,
            }
        );

        let num_instances_per_row: u32 = 1;
        let instance_displacement: cgmath::Vector3<f32> = cgmath::Vector3::new(num_instances_per_row as f32 * 0.0, 0.0, num_instances_per_row as f32 * 0.0);


        let instances = (0..num_instances_per_row).flat_map(|z| {
            (0..num_instances_per_row).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - instance_displacement;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Entity { vertex_buffer, index_buffer, instance_buffer, instances, num_indices: indicies.len() as u32, diffuse_bind_group: None, diffuse_texture: None }
    }

    pub(crate) fn new(device: &mut wgpu::Device , vertex_data: &VertexIndicie, num_instances_per_row: u32, instance_displacement: cgmath::Vector3<f32>) -> Entity {
         let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data.vertexes),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        

        let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&vertex_data.indicies),
            usage: wgpu::BufferUsages::INDEX,
            }
        );




        let instances = (0..num_instances_per_row).flat_map(|z| {
            (0..num_instances_per_row).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - instance_displacement;

                let rotation = if position.is_zero() {
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        Entity { vertex_buffer, index_buffer, instance_buffer, instances, num_indices: vertex_data.indicies.len() as u32, diffuse_bind_group: None, diffuse_texture: None }
    }
}

pub enum EntityType {
    Rectangle,
    Circle,
    VertIndicie
}

pub (crate) struct VertexIndicie {
    pub (crate) vertexes: Vec<Vertex>,
    pub (crate) indicies: Vec<u16>,
    pub (crate) entity_type: EntityType 
}

pub struct Renderer {
    pub (crate) entity_vertex_data: Vec<VertexIndicie>,
    pub (crate) screen_width: u32,
    pub (crate) screen_height: u32
}

impl Renderer {
    pub fn draw_circle(&self, location: Point2D, radius: u32) {
        
    }

    pub fn draw_rectangle(&mut self, location: Point2D, width: u32, height: u32) {
        let top_right = Vertex { position: [location.x + 1.0 / width as f32, location.y, 0.0], tex_coords: [0.0, 0.0]};
        let top_left = Vertex { position: [location.x, location.y, 0.0], tex_coords: [0.0, 0.0]};
        let bottom_left = Vertex { position: [location.x, location.y - (1.0 / height as f32), 0.0], tex_coords: [0.0, 0.0]};
        let bottom_right = Vertex { position: [location.x + 1.0 / width as f32, location.y - (1.0 / height as f32), 0.0], tex_coords: [0.0, 0.0]};
        let entity_vertex_data = VertexIndicie { vertexes: vec![top_left, top_right, bottom_left, bottom_right], indicies: vec![0, 1, 2, 2, 3, 1], entity_type: EntityType::Rectangle};
        self.entity_vertex_data.push(entity_vertex_data);
    }
}
