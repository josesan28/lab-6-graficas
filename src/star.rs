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

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let time = uniforms.time * FLOW_SPEED;
    let drift = Vector3::new(time * 0.3, time, -time * 0.4);

    let flow_position = p * 3.0 + drift;
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

    let cells = sample(&uniforms.noise, surface * CELL_FREQUENCY + drift).abs();
    let heat = smoothstep(0.03, 0.48, cells);
    let orange = Vector3::new(1.0, 0.27, 0.025);
    let gold = Vector3::new(1.0, 0.75, 0.12);
    let cream = Vector3::new(1.0, 0.96, 0.58);
    let color = mix(orange, gold, smoothstep(0.0, 0.65, heat));
    let color = mix(color, cream, smoothstep(0.55, 1.0, heat));

    let activity = fbm(&uniforms.noise, p * 3.5 - drift * 0.45);
    let spots = smoothstep(SPOT_THRESHOLD, SPOT_THRESHOLD + 0.13, activity);
    let color = mix(color, Vector3::new(0.62, 0.09, 0.015), spots * 0.85);

    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let facing = fragment.normal.dot(view).max(0.0);
    let edge = (1.0 - facing).powf(3.0);
    mix(color, Vector3::new(1.0, 0.36, 0.035), edge * 0.5)
}
