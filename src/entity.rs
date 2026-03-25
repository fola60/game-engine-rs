use wgpu::util::DeviceExt;
use cgmath::{InnerSpace, Rotation3, Zero};
use crate::{
    Instance, renderer::VertexIndicie, texture::Texture
};

pub struct Entity {
    pub(crate) vertex_data: VertexIndicie,
    pub(crate) num_instances: u32,
    pub(crate) instance_displacement: cgmath::Vector3<f32>,
    pub(crate) diffuse_bind_group: Option<wgpu::BindGroup>,
    pub(crate) diffuse_texture: Option<Texture>
}

impl Entity {
    pub(crate) fn get_vertex_buffer(&self, device: &mut wgpu::Device) -> wgpu::Buffer {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.vertex_data.vertexes),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );
        vertex_buffer
    }

    pub(crate) fn get_index_buffer(&self, device: &mut wgpu::Device) -> wgpu::Buffer {
        let index_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.vertex_data.indicies),
            usage: wgpu::BufferUsages::INDEX,
            }
        );
        index_buffer
    }

    pub(crate) fn get_instance_buffer(&self, device: &mut wgpu::Device) -> wgpu::Buffer {

        let instances = (0..self.num_instances).flat_map(|z| {
            (0..self.num_instances).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - &self.instance_displacement;

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
        instance_buffer
    }

    pub (crate) fn new_rectangle(device: &mut wgpu::Device , vertex_data: VertexIndicie) -> Entity {
        let num_instances_per_row: u32 = 1;
        let instance_displacement: cgmath::Vector3<f32> = cgmath::Vector3::new(num_instances_per_row as f32 * 0.0, 0.0, num_instances_per_row as f32 * 0.0);
        Entity { vertex_data, num_instances: num_instances_per_row, instance_displacement, diffuse_bind_group: None, diffuse_texture: None }
    }

    pub(crate) fn new(vertex_data: VertexIndicie, num_instances_per_row: u32, instance_displacement: cgmath::Vector3<f32>) -> Entity {
         
        Entity {vertex_data, num_instances: num_instances_per_row, instance_displacement, diffuse_bind_group: None, diffuse_texture: None}
    }
}
