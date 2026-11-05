use crate::{Uniforms, fragment::Fragment, pipeline::lighting, vertex::RasterVertex};
use raylib::prelude::*;

pub fn triangle(vertices: &[RasterVertex], uniforms: &Uniforms) -> Vec<Fragment> {
    let a = &vertices[0];
    let b = &vertices[1];
    let c = &vertices[2];
    let p = a.screen_position;
    let q = b.screen_position;
    let r = c.screen_position;
    let denominator = (q.y - r.y) * (p.x - r.x) + (r.x - q.x) * (p.y - r.y);
    if denominator.abs() < 0.0001 || vertices.iter().any(|v| v.inverse_w <= 0.0) {
        return Vec::new();
    }
    let min_x = p.x.min(q.x).min(r.x).floor().max(0.0) as i32;
    let max_x =
        p.x.max(q.x)
            .max(r.x)
            .ceil()
            .min(uniforms.resolution.x - 1.0) as i32;
    let min_y = p.y.min(q.y).min(r.y).floor().max(0.0) as i32;
    let max_y =
        p.y.max(q.y)
            .max(r.y)
            .ceil()
            .min(uniforms.resolution.y - 1.0) as i32;
    let face_normal = (b.world_position - a.world_position)
        .cross(c.world_position - a.world_position)
        .normalized();
    let center = (a.world_position + b.world_position + c.world_position) / 3.0;
    let flat_intensity = lighting(face_normal, center, uniforms.light_position);
    let mut fragments = Vec::new();
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let sx = x as f32 + 0.5;
            let sy = y as f32 + 0.5;
            let w0 = ((q.y - r.y) * (sx - r.x) + (r.x - q.x) * (sy - r.y)) / denominator;
            let w1 = ((r.y - p.y) * (sx - r.x) + (p.x - r.x) * (sy - r.y)) / denominator;
            let w2 = 1.0 - w0 - w1;
            if w0 < -0.00001 || w1 < -0.00001 || w2 < -0.00001 {
                continue;
            }
            let depth = p.z * w0 + q.z * w1 + r.z * w2;
            if !(-1.0..=1.0).contains(&depth) {
                continue;
            }
            let reciprocal = a.inverse_w * w0 + b.inverse_w * w1 + c.inverse_w * w2;
            let u = w0 * a.inverse_w / reciprocal;
            let v = w1 * b.inverse_w / reciprocal;
            let w = w2 * c.inverse_w / reciprocal;
            let normal = (a.normal * u + b.normal * v + c.normal * w).normalized();
            let world_position = a.world_position * u + b.world_position * v + c.world_position * w;
            fragments.push(Fragment {
                position: Vector2::new(sx, sy),
                object_position: a.object_position * u
                    + b.object_position * v
                    + c.object_position * w,
                world_position,
                normal,
                face_normal,
                uv: a.uv * u + b.uv * v + c.uv * w,
                color: Vector3::new(1.0, 1.0, 1.0),
                depth,
                intensity: lighting(normal, world_position, uniforms.light_position),
                flat_intensity,
                gouraud_intensity: a.intensity * w0 + b.intensity * w1 + c.intensity * w2,
            });
        }
    }
    fragments
}
