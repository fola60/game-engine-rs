//! Immutable CPU meshes shared by user entities and uploaded by the engine.
use crate::{Dimension, ThreeD, TwoD, Z, model::ModelVertex, renderer::VertexIndicie, world_units};
use std::{marker::PhantomData, sync::Arc};

/// A cheap-to-clone immutable mesh. Create once and reuse in render_data.
pub struct Mesh<D: Dimension> {
    pub(crate) data: Arc<VertexIndicie>,
    dimension: PhantomData<D>,
}

impl<D: Dimension> Clone for Mesh<D> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            dimension: PhantomData,
        }
    }
}

impl<D: Dimension> Mesh<D> {
    /// Vertices are in world units; indices describe triangles.
    pub fn new(vertices: Vec<ModelVertex>, indices: Vec<u16>) -> anyhow::Result<Self> {
        anyhow::ensure!(!vertices.is_empty(), "a mesh must have vertices");
        anyhow::ensure!(
            !indices.is_empty() && indices.len() % 3 == 0,
            "indices must describe complete triangles"
        );
        anyhow::ensure!(
            indices.iter().all(|&i| (i as usize) < vertices.len()),
            "mesh index out of bounds"
        );
        anyhow::ensure!(
            vertices.iter().all(|v| v
                .position
                .iter()
                .chain(v.tex_coords.iter())
                .chain(v.normal.iter())
                .all(|x| x.is_finite())),
            "mesh vertices must be finite"
        );
        Ok(Self::from_data(VertexIndicie {
            vertexes: vertices,
            indicies: indices,
        }))
    }

    fn from_data(data: VertexIndicie) -> Self {
        Self {
            data: Arc::new(data),
            dimension: PhantomData,
        }
    }

    pub fn vertices(&self) -> &[ModelVertex] {
        &self.data.vertexes
    }
    pub fn indices(&self) -> &[u16] {
        &self.data.indicies
    }
}

impl Mesh<TwoD> {
    pub fn circle(radius: f32) -> Self {
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
        };
        Self::from_data(data)
    }
    pub fn rectangle(width: f32, height: f32) -> Self {
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
        };
        Self::from_data(entity_vertex_data)
    }
}
impl Mesh<ThreeD> {
    pub fn cube(width: f32, height: f32, length: f32) -> Self {
        let half_w = world_units::meters_to_world(width) * 0.5;
        let half_h = world_units::meters_to_world(height) * 0.5;
        let half_l = world_units::meters_to_world(length) * 0.5;

        let vertices = vec![
            // Front (+Z)
            crate::model::ModelVertex {
                position: [-half_w, -half_h, half_l],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            crate::model::ModelVertex {
                position: [half_w, -half_h, half_l],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, 1.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, half_l],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, half_h, half_l],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, 1.0],
            },
            // Back (-Z)
            crate::model::ModelVertex {
                position: [half_w, -half_h, -half_l],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, -half_h, -half_l],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 0.0, -1.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, half_h, -half_l],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, -half_l],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 0.0, -1.0],
            },
            // Left (-X)
            crate::model::ModelVertex {
                position: [-half_w, -half_h, -half_l],
                tex_coords: [0.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, -half_h, half_l],
                tex_coords: [1.0, 1.0],
                normal: [-1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, half_h, half_l],
                tex_coords: [1.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, half_h, -half_l],
                tex_coords: [0.0, 0.0],
                normal: [-1.0, 0.0, 0.0],
            },
            // Right (+X)
            crate::model::ModelVertex {
                position: [half_w, -half_h, half_l],
                tex_coords: [0.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, -half_h, -half_l],
                tex_coords: [1.0, 1.0],
                normal: [1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, -half_l],
                tex_coords: [1.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, half_l],
                tex_coords: [0.0, 0.0],
                normal: [1.0, 0.0, 0.0],
            },
            // Top (+Y)
            crate::model::ModelVertex {
                position: [-half_w, half_h, half_l],
                tex_coords: [0.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, half_l],
                tex_coords: [1.0, 1.0],
                normal: [0.0, 1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, half_h, -half_l],
                tex_coords: [1.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, half_h, -half_l],
                tex_coords: [0.0, 0.0],
                normal: [0.0, 1.0, 0.0],
            },
            // Bottom (-Y)
            crate::model::ModelVertex {
                position: [-half_w, -half_h, -half_l],
                tex_coords: [0.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, -half_h, -half_l],
                tex_coords: [1.0, 1.0],
                normal: [0.0, -1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [half_w, -half_h, half_l],
                tex_coords: [1.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            },
            crate::model::ModelVertex {
                position: [-half_w, -half_h, half_l],
                tex_coords: [0.0, 0.0],
                normal: [0.0, -1.0, 0.0],
            },
        ];

        let indicies: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3, // front
            4, 5, 6, 4, 6, 7, // back
            8, 9, 10, 8, 10, 11, // left
            12, 13, 14, 12, 14, 15, // right
            16, 17, 18, 16, 18, 19, // top
            20, 21, 22, 20, 22, 23, // bottom
        ];

        let entity_vertex_data = VertexIndicie {
            vertexes: vertices,
            indicies,
        };
        Self::from_data(entity_vertex_data)
    }
}
