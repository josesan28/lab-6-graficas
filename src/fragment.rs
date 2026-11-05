#![allow(dead_code)]

use raylib::prelude::*;

pub struct Fragment {
    pub position: Vector2,
    pub object_position: Vector3,
    pub world_position: Vector3,
    pub normal: Vector3,
    pub face_normal: Vector3,
    pub uv: Vector2,
    pub color: Vector3,
    pub depth: f32,
    pub intensity: f32,
    pub flat_intensity: f32,
    pub gouraud_intensity: f32,
}
