use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
    surface::Surface,
};
use raylib::prelude::*;
use std::sync::OnceLock;

const DEPTH: f32 = 0.075;
const STEPS: usize = 6;
static CRUST: OnceLock<Surface<6>> = OnceLock::new();

fn terrain(p: Vector3, u: &Uniforms) -> [f32; 3] {
    let warp = sample(&u.noise, p * 3.0);
    let fissure = sample(&u.noise, p * 8.0 + Vector3::new(warp, warp * 0.3, 2.0));
    let cracks = 1.0 - smoothstep(0.025, 0.14, fissure.abs());
    let grain = fbm(&u.noise, p * 46.0);
    let height = (1.0 - cracks) * 0.65 + grain * 0.25;
    [height, cracks, grain]
}

pub fn prepare(u: &Uniforms) {
    CRUST.get_or_init(|| {
        Surface::new(|p| {
            let [height, cracks, grain] = terrain(p, u);
            let e = 0.003;
            let gradient = Vector3::new(
                terrain(p + Vector3::new(e, 0.0, 0.0), u)[0] - height,
                terrain(p + Vector3::new(0.0, e, 0.0), u)[0] - height,
                terrain(p + Vector3::new(0.0, 0.0, e), u)[0] - height,
            ) / e;
            [height, cracks, grain, gradient.x, gradient.y, gradient.z]
        })
    });
}

pub fn displaced(
    p: Vector3,
    view: Vector3,
    enabled: bool,
    height: impl Fn(Vector3) -> f32,
) -> Vector3 {
    if !enabled {
        return p;
    }
    let n = p.normalized();
    let tangent = view - n * view.dot(n);
    let facing = view.dot(n).abs();
    let ray = tangent * (DEPTH * smoothstep(0.12, 0.55, facing) / facing.max(0.35));
    let mut previous = p;
    let mut previous_gap = -(1.0 - height(p));
    for step in 1..=STEPS {
        let depth = step as f32 / STEPS as f32;
        let current = p - ray * depth;
        let gap = depth - (1.0 - height(current));
        if gap >= 0.0 {
            let blend = (-previous_gap / (gap - previous_gap).max(0.00001)).clamp(0.0, 1.0);
            return previous + (current - previous) * blend;
        }
        previous = current;
        previous_gap = gap;
    }
    previous
}

pub fn shade(f: &Fragment, u: &Uniforms) -> Vector3 {
    let map = CRUST.get().expect("Corteza preparada");
    let view = (u.camera_position - f.world_position).normalized();
    let object_view = crate::patterns::object_direction(view, u);
    let p = displaced(
        f.object_position.normalized(),
        object_view,
        u.parallax_enabled,
        |p| map.sample(p)[0],
    );
    let [_, cracks, grain, gx, gy, gz] = map.sample(p);
    let gradient = crate::patterns::world_direction(Vector3::new(gx, gy, gz), u);
    let n = if u.bump_enabled {
        (f.normal - (gradient - f.normal * gradient.dot(f.normal)) * 0.025).normalized()
    } else {
        f.normal
    };
    let light = (u.light_position - f.world_position).normalized();
    let diffuse = n.dot(light).max(0.0);
    let basalt = mix(
        Vector3::new(0.065, 0.025, 0.12),
        Vector3::new(0.30, 0.16, 0.40),
        grain,
    );
    let pulse = 0.84 + 0.16 * (u.time * 1.7 + p.y * 11.0 + grain * 5.0).sin();
    let lava = mix(
        Vector3::new(1.0, 0.12, 0.008),
        Vector3::new(1.0, 0.85, 0.16),
        smoothstep(0.4, 1.0, cracks),
    );
    let surface = mix(basalt * (0.22 + diffuse * 0.78), lava * pulse, cracks);
    let edge = (1.0 - f.normal.dot(view).max(0.0)).powi(4);
    surface + Vector3::new(0.34, 0.055, 0.012) * edge
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parallax_moves_with_view_and_preserves_disabled_coordinates() {
        let p = Vector3::new(0.0, 0.0, 1.0);
        let view = Vector3::new(0.6, 0.0, 0.8);
        assert_eq!(displaced(p, view, false, |_| 0.5), p);
        assert!((displaced(p, p, true, |_| 0.5) - p).length() < 0.00001);
        let a = displaced(p, view, true, |_| 0.5);
        let b = displaced(p, Vector3::new(-0.6, 0.0, 0.8), true, |_| 0.5);
        assert!(a.x < 0.0 && b.x > 0.0);
        assert!((a.x + b.x).abs() < 0.00001);
    }
}
