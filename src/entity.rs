use crate::{
    Instance, InstanceRaw, Transform,
    model::Model,
    renderer::{EntityType, VertexIndicie},
    texture::Texture,
};
use cgmath::{InnerSpace, Rotation3, Zero};
use image::GenericImageView;
use wgpu::util::DeviceExt;

pub struct Entity {
    pub(crate) id: u32,
    pub(crate) vertex_data: VertexIndicie,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) num_instances: u32,
    pub(crate) instance_displacement: cgmath::Vector3<f32>,
    pub(crate) diffuse_texture_name: Option<String>,
    pub(crate) texture: Option<Texture>,
    pub(crate) model: Option<Model>,
    pub(crate) transform: Transform,
    pub(crate) color: [f32; 4],
    pub(crate) instance_dirty: bool,
}

impl Entity {
    fn build_vertex_buffer(device: &wgpu::Device, vertex_data: &VertexIndicie) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data.vertexes),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn build_index_buffer(device: &wgpu::Device, vertex_data: &VertexIndicie) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&vertex_data.indicies),
            usage: wgpu::BufferUsages::INDEX,
        })
    }

    fn build_instances(
        num_instances: u32,
        instance_displacement: cgmath::Vector3<f32>,
        transform: Transform,
        color: [f32; 4],
    ) -> Vec<InstanceRaw> {
        (0..num_instances)
            .flat_map(|z| {
                (0..num_instances).map(move |x| {
                    let position = cgmath::Vector3 {
                        x: x as f32,
                        y: 0.0,
                        z: z as f32,
                    } - instance_displacement;

                    let rotation = if position.is_zero() {
                        // this is needed so an object at (0, 0, 0) won't get scaled to zero
                        // as Quaternions can affect scale if they're not created correctly
                        cgmath::Quaternion::from_axis_angle(
                            cgmath::Vector3::unit_z(),
                            cgmath::Deg(0.0),
                        )
                    } else {
                        cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                    };

                    Instance {
                        model: transform.matrix()
                            * cgmath::Matrix4::from_translation(position)
                            * cgmath::Matrix4::from(rotation),
                        color: color.into(),
                    }
                    .to_raw()
                })
            })
            .collect()
    }

    fn build_instance_buffer(
        device: &wgpu::Device,
        num_instances: u32,
        instance_displacement: cgmath::Vector3<f32>,
        transform: Transform,
        color: [f32; 4],
    ) -> wgpu::Buffer {
        let instances =
            Self::build_instances(num_instances, instance_displacement, transform, color);
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub(crate) fn sync_instance_buffer(&mut self, queue: &wgpu::Queue) {
        if !self.instance_dirty {
            return;
        }

        let instances = Self::build_instances(
            self.num_instances,
            self.instance_displacement,
            self.transform,
            self.color,
        );
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
        self.instance_dirty = false;
    }

    pub fn set_material_color(&self, queue: &mut wgpu::Queue, material_buffer: &mut wgpu::Buffer) {
        queue.write_buffer(material_buffer, 0, bytemuck::cast_slice(&[self.color]));
    }

    pub fn set_diffuse(&self, queue: &wgpu::Queue, device: &wgpu::Device) {
        let diffuse_bytes = include_bytes!("../assets/happy-tree.png");
        let img = image::load_from_memory(diffuse_bytes).unwrap();
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("bluh"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

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

    pub(crate) fn new(
        id: u32,
        vertex_data: VertexIndicie,
        num_instances_per_row: u32,
        instance_displacement: cgmath::Vector3<f32>,
        device: &wgpu::Device,
    ) -> Entity {
        let transform = Transform::default();
        let color = [1.0, 1.0, 1.0, 1.0];
        let vertex_buffer = Self::build_vertex_buffer(device, &vertex_data);
        let index_buffer = Self::build_index_buffer(device, &vertex_data);
        let instance_buffer = Self::build_instance_buffer(
            device,
            num_instances_per_row,
            instance_displacement,
            transform,
            color,
        );

        Entity {
            id,
            vertex_data,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_instances: num_instances_per_row,
            instance_displacement,
            diffuse_texture_name: None,
            texture: None,
            model: None,
            transform,
            color,
            instance_dirty: false,
        }
    }

    pub(crate) fn from_model(
        id: u32,
        model: Model,
        instance_displacement: cgmath::Vector3<f32>,
        device: &wgpu::Device,
    ) -> Entity {
        let vertex_data = VertexIndicie {
            vertexes: vec![crate::model::ModelVertex {
                position: [0.0, 0.0, 0.0],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            }],
            indicies: vec![0],
        };

        let mut entity = Self::new(id, vertex_data, 1, instance_displacement, device);
        entity.model = Some(model);
        entity
    }
}
