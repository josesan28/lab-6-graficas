use crate::{
    Uniforms,
    vertex::{RasterVertex, Vertex},
};
use raylib::prelude::*;

pub fn transform(matrix: &Matrix, vector: Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

pub fn lighting(normal: Vector3, position: Vector3, light: Vector3) -> f32 {
    normal.dot((light - position).normalized()).max(0.0)
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> RasterVertex {
    let p = vertex.position;
    let world = transform(&uniforms.model_matrix, Vector4::new(p.x, p.y, p.z, 1.0));
    let view = transform(&uniforms.view_matrix, world);
    let clip = transform(&uniforms.projection_matrix, view);
    let inverse_w = 1.0 / clip.w;
    let screen = transform(
        &uniforms.viewport_matrix,
        Vector4::new(
            clip.x * inverse_w,
            clip.y * inverse_w,
            clip.z * inverse_w,
            1.0,
        ),
    );
    let n = if uniforms.smooth_normals {
        vertex.smooth_normal
    } else {
        vertex.normal
    };
    let normal = transform(&uniforms.model_matrix, Vector4::new(n.x, n.y, n.z, 0.0));
    let normal = Vector3::new(normal.x, normal.y, normal.z).normalized();
    let world_position = Vector3::new(world.x, world.y, world.z);
    RasterVertex {
        screen_position: Vector3::new(screen.x, screen.y, screen.z),
        object_position: p,
        world_position,
        normal,
        uv: vertex.tex_coords,
        inverse_w,
        intensity: lighting(normal, world_position, uniforms.light_position),
    }
}
