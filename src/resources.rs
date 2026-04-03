use std::path::Path;

use wgpu::util::DeviceExt;

use crate::{model, texture, world_units};

pub fn load_string(file_name: &str) -> anyhow::Result<String> {
    Ok(std::fs::read_to_string(file_name)?)
}

pub fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    Ok(std::fs::read(file_name)?)
}

pub fn load_texture(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> anyhow::Result<texture::Texture> {
    let data = load_binary(file_name)?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub fn load_model(
    file_name: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> anyhow::Result<model::Model> {
    let obj_parent = Path::new(file_name)
        .parent()
        .unwrap_or_else(|| Path::new("."));

    let (models, obj_materials) = tobj::load_obj(
        file_name,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )?;

    let mut materials = Vec::new();
    for m in obj_materials.unwrap_or_default() {
        let diffuse_texture = if let Some(diffuse_name) = m.diffuse_texture {
            let diffuse_path = obj_parent.join(diffuse_name);
            match load_texture(diffuse_path.to_string_lossy().as_ref(), device, queue) {
                Ok(texture) => texture,
                Err(_) => texture::Texture::default(device, queue)?,
            }
        } else {
            texture::Texture::default(device, queue)?
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: None,
        });

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        });
    }

    if materials.is_empty() {
        let diffuse_texture = texture::Texture::default(device, queue)?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some("default_model_material_bind_group"),
        });
        materials.push(model::Material {
            name: String::from("default"),
            diffuse_texture,
            bind_group,
        });
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| {
                    let tex_coords = if m.mesh.texcoords.is_empty() {
                        [0.0, 0.0]
                    } else {
                        [m.mesh.texcoords[i * 2], 1.0 - m.mesh.texcoords[i * 2 + 1]]
                    };

                    if m.mesh.normals.is_empty() {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3] * world_units::MODEL_IMPORT_SCALE,
                                m.mesh.positions[i * 3 + 1] * world_units::MODEL_IMPORT_SCALE,
                                m.mesh.positions[i * 3 + 2] * world_units::MODEL_IMPORT_SCALE,
                            ],
                            tex_coords,
                            normal: [0.0, 0.0, 0.0],
                        }
                    } else {
                        model::ModelVertex {
                            position: [
                                m.mesh.positions[i * 3] * world_units::MODEL_IMPORT_SCALE,
                                m.mesh.positions[i * 3 + 1] * world_units::MODEL_IMPORT_SCALE,
                                m.mesh.positions[i * 3 + 2] * world_units::MODEL_IMPORT_SCALE,
                            ],
                            tex_coords,
                            normal: [
                                m.mesh.normals[i * 3],
                                m.mesh.normals[i * 3 + 1],
                                m.mesh.normals[i * 3 + 2],
                            ],
                        }
                    }
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            model::Mesh {
                name: m.name,
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
