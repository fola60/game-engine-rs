pub const WORLD_UNITS_PER_METER: f32 = 1.0;
pub const METERS_PER_WORLD_UNIT: f32 = 1.0 / WORLD_UNITS_PER_METER;

pub const ORTHOGRAPHIC_VIEW_HEIGHT_METERS: f32 = 20.0;
pub const CAMERA_3D_DISTANCE_METERS: f32 = 5.0;
pub const CAMERA_3D_FOVY_DEGREES: f32 = 45.0;
pub const CAMERA_3D_ZNEAR_METERS: f32 = 0.1;
pub const CAMERA_3D_ZFAR_METERS: f32 = 100.0;
pub const CAMERA_2D_ZNEAR_METERS: f32 = -100.0;
pub const CAMERA_2D_ZFAR_METERS: f32 = 100.0;

pub const MODEL_IMPORT_SCALE: f32 = 1.0;

#[inline]
pub const fn meters_to_world(meters: f32) -> f32 {
    meters * WORLD_UNITS_PER_METER
}

#[inline]
pub const fn world_to_meters(world_units: f32) -> f32 {
    world_units * METERS_PER_WORLD_UNIT
}
