use raylib::prelude::*;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub smooth_normal: Vector3,
    pub tex_coords: Vector2,
}

#[derive(Clone)]
pub struct RasterVertex {
    pub screen_position: Vector3,
    pub world_position: Vector3,
    pub object_position: Vector3,
    pub normal: Vector3,
    pub uv: Vector2,
    pub inverse_w: f32,
    pub intensity: f32,
}
