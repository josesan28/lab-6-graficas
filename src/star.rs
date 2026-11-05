use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
};
use raylib::prelude::*;

const FLOW_SPEED: f32 = 0.18;
const CELL_FREQUENCY: f32 = 9.0;
const WARP_STRENGTH: f32 = 0.35;
const SPOT_THRESHOLD: f32 = 0.64;

static PLASMA: std::sync::OnceLock<crate::surface::Surface<3>> = std::sync::OnceLock::new();
pub fn prepare(uniforms: &Uniforms) {
    PLASMA.get_or_init(|| {
        crate::surface::Surface::new(|direction| {
            let color = material(direction * 0.8660254, uniforms);
            [color.x, color.y, color.z]
        })
    });
}
fn material(p: Vector3, uniforms: &Uniforms) -> Vector3 {
    let flow_position = p * 3.0;
    let warp = Vector3::new(
        sample(&uniforms.noise, flow_position),
        sample(
            &uniforms.noise,
            flow_position + Vector3::new(17.0, 5.0, 3.0),
        ),
        sample(
            &uniforms.noise,
            flow_position + Vector3::new(2.0, 23.0, 11.0),
        ),
    );
    let surface = p + warp * WARP_STRENGTH;

    let cells = sample(&uniforms.noise, surface * CELL_FREQUENCY).abs();
    let heat = smoothstep(0.04, 0.40, cells);
    let orange = Vector3::new(1.0, 0.27, 0.025);
    let gold = Vector3::new(1.0, 0.75, 0.12);
    let cream = Vector3::new(1.0, 0.96, 0.58);
    let color = mix(orange, gold, smoothstep(0.12, 0.52, heat));
    let color = mix(color, cream, smoothstep(0.70, 0.93, heat));

    let activity = fbm(&uniforms.noise, p * 3.5);
    let spots = smoothstep(SPOT_THRESHOLD, SPOT_THRESHOLD + 0.13, activity);
    let color = mix(color, Vector3::new(0.62, 0.09, 0.015), spots * 0.85);
    // El ruido de mayor frecuencia añade variación dentro de las celdas.
    let grain = sample(&uniforms.noise, surface * 78.0).abs();
    let granules = smoothstep(0.03, 0.34, grain);
    let filaments = (1.0 - smoothstep(0.015, 0.09, cells)) * 0.28;
    let color = color * (0.72 + granules * 0.28) + Vector3::new(0.30, 0.065, 0.006) * filaments;
    let core = smoothstep(SPOT_THRESHOLD + 0.07, SPOT_THRESHOLD + 0.14, activity);
    mix(color, Vector3::new(0.16, 0.025, 0.009), core * 0.8)
}
pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let t = uniforms.time * FLOW_SPEED;
    // Advección no rígida: las celdas se deforman, no gira la geometría.
    let flow = Vector3::new(
        (p.y * 7.0 + t).sin(),
        (p.z * 6.0 - t * 0.7).sin(),
        (p.x * 7.0 + t * 0.8).cos(),
    );
    let surface = p + flow * 0.025;
    let [r, g, b] = PLASMA.get().expect("Plasma preparado").sample(surface);
    let color = Vector3::new(r, g, b);
    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let facing = fragment.normal.dot(view).max(0.0);
    let edge = (1.0 - facing).powi(3);
    mix(color, Vector3::new(1.0, 0.36, 0.035), edge * 0.5)
}
