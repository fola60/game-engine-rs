use crate::{Dimension, RenderData};
use cgmath::{Vector3, Vector4};

pub(crate) struct Bounds {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl Bounds {
    pub(crate) fn from_render_data<D: Dimension>(data: &RenderData<D>) -> Option<Self> {
        let matrix = data.transform.matrix();
        let mut bounds = Self {
            min: Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
        };
        if data.mesh.indices().is_empty() {
            return None;
        }
        for &index in data.mesh.indices() {
            let [x, y, z] = data.mesh.vertices()[index as usize].position;
            let point = matrix * Vector4::new(x, y, z, 1.0);
            for axis in 0..3 {
                if !point[axis].is_finite() {
                    return None;
                }
                bounds.min[axis] = bounds.min[axis].min(point[axis]);
                bounds.max[axis] = bounds.max[axis].max(point[axis]);
            }
        }
        Some(bounds)
    }

    pub(crate) fn overlaps(&self, other: &Self) -> bool {
        (0..3).all(|axis| self.min[axis] <= other.max[axis] && other.min[axis] <= self.max[axis])
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollisionPhase {
    Started,
    Stayed,
    Ended,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionEvent {
    pub other_id: String,
    pub phase: CollisionPhase,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneCollisionEvent {
    pub a_id: String,
    pub b_id: String,
    pub phase: CollisionPhase,
}

impl SceneCollisionEvent {
    pub fn involves(&self, a: &str, b: &str) -> bool {
        (self.a_id == a && self.b_id == b) || (self.a_id == b && self.b_id == a)
    }

    pub fn other_id(&self, id: &str) -> Option<&str> {
        if self.a_id == id {
            Some(&self.b_id)
        } else if self.b_id == id {
            Some(&self.a_id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Mesh, ThreeD, TwoD};
    use cgmath::{Deg, Quaternion, Rotation3};

    #[test]
    fn mesh_bounds_include_transforms_and_require_overlap_on_every_axis() {
        let mut data = RenderData::new(Mesh::<ThreeD>::cube(2.0, 4.0, 6.0));
        data.transform.position = Vector3::new(5.0, 6.0, 7.0);
        data.transform.scale = Vector3::new(-2.0, 0.5, 1.0);
        data.transform.rotation = Quaternion::from_angle_z(Deg(90.0));
        let bounds = Bounds::from_render_data(&data).unwrap();
        for axis in 0..3 {
            assert!((bounds.min[axis] - 4.0).abs() < 0.00001);
            assert!((bounds.max[axis] - [6.0, 8.0, 10.0][axis]).abs() < 0.00001);
        }
        let mut cube = RenderData::new(Mesh::<ThreeD>::cube(1.0, 1.0, 1.0));
        let origin = Bounds::from_render_data(&cube).unwrap();
        for axis in 0..3 {
            cube.transform.position = Vector3::new(0.0, 0.0, 0.0);
            cube.transform.position[axis] = 1.0;
            assert!(origin.overlaps(&Bounds::from_render_data(&cube).unwrap()));
            cube.transform.position[axis] = 1.01;
            assert!(!origin.overlaps(&Bounds::from_render_data(&cube).unwrap()));
        }
        cube.transform.position.x = f32::NAN;
        assert!(Bounds::from_render_data(&cube).is_none());
        let flat =
            Bounds::from_render_data(&RenderData::new(Mesh::<TwoD>::rectangle(2.0, 4.0))).unwrap();
        assert!(flat.overlaps(&flat));
        assert_eq!(flat.min.z, flat.max.z);
    }
}
