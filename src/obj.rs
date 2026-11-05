use crate::vertex::Vertex;
use raylib::prelude::*;

pub fn load(path: &str) -> Result<Vec<Vertex>, tobj::LoadError> {
    let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
    let mut vertices = Vec::new();
    for model in models {
        let mesh = model.mesh;
        for index in mesh.indices {
            let index = index as usize;
            let position = Vector3::new(
                mesh.positions[index * 3],
                mesh.positions[index * 3 + 1],
                mesh.positions[index * 3 + 2],
            );
            let normal = if mesh.normals.is_empty() {
                position.normalized()
            } else {
                Vector3::new(
                    mesh.normals[index * 3],
                    mesh.normals[index * 3 + 1],
                    mesh.normals[index * 3 + 2],
                )
            };
            let tex_coords = if mesh.texcoords.is_empty() {
                Vector2::zero()
            } else {
                Vector2::new(mesh.texcoords[index * 2], mesh.texcoords[index * 2 + 1])
            };
            vertices.push(Vertex {
                position,
                normal,
                smooth_normal: Vector3::zero(),
                tex_coords,
            });
        }
    }
    let mut normals = std::collections::HashMap::new();
    let key = |p: Vector3| (p.x.to_bits(), p.y.to_bits(), p.z.to_bits());
    for face in vertices.chunks_exact(3) {
        let normal =
            (face[1].position - face[0].position).cross(face[2].position - face[0].position);
        for vertex in face {
            *normals
                .entry(key(vertex.position))
                .or_insert(Vector3::zero()) += normal;
        }
    }
    for vertex in &mut vertices {
        vertex.smooth_normal = normals[&key(vertex.position)].normalized();
    }
    Ok(vertices)
}
