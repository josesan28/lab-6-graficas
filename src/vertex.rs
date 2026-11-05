use raylib::prelude::*;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub smooth_normal: Vector3,
}

#[derive(Clone)]
pub struct RasterVertex {
    pub screen_position: Vector3,
    pub world_position: Vector3,
    pub object_position: Vector3,
    pub normal: Vector3,
    pub inverse_w: f32,
}
