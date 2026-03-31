use image::GenericImageView;
use wgpu::{util::DeviceExt};
use cgmath::{InnerSpace, Rotation3, Vector3, Zero};
use crate::{
    Instance, Vertex, renderer::VertexIndicie, texture::Texture
};

pub struct Entity {
    pub(crate) id: u32,
    pub(crate) vertex_data: VertexIndicie,
    pub(crate) num_instances: u32,
    pub(crate) instance_displacement: cgmath::Vector3<f32>,
    pub(crate) diffuse_texture_name: Option<String>,
    pub(crate) texture: Option<Texture>,
    pub(crate) location: [f32;3],
    pub(crate) color: [f32; 4]
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
        let location = self.location;
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
                    position, rotation, vertex_offset: location.into(), color: self.color.into()
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

    
    
    pub fn set_material_color(&self, queue: &mut wgpu::Queue, material_buffer: &mut wgpu::Buffer) {
        queue.write_buffer(
            material_buffer,
        0,
        bytemuck::cast_slice(&[self.color]),
        );
    }

    pub fn set_diffuse(&self, queue: &wgpu::Queue, device: &mut wgpu::Device) {
        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let img = image::load_from_memory(diffuse_bytes).unwrap();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("bluh"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            }
        );

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture, 
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1,
            },
        );
    }

    pub(crate) fn new(id: u32, vertex_data: VertexIndicie, num_instances_per_row: u32, instance_displacement: cgmath::Vector3<f32>) -> Entity {
        Entity {
            id, 
            vertex_data, 
            num_instances: num_instances_per_row, 
            instance_displacement, 
            diffuse_texture_name: None, 
            texture: None,
            location: [0.0, 0.0, 0.0], 
            color: [0.0, 0.0, 0.0, 0.0]
        }
    }
}
