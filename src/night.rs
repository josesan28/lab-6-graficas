use crate::{Uniforms, fragment::Fragment, patterns::smoothstep};
use raylib::prelude::*;

const LIGHT_STRENGTH: f32 = 1.4;

pub fn emission(
    fragment: &Fragment,
    uniforms: &Uniforms,
    land_mask: f32,
    height: f32,
    cities: f32,
) -> Vector3 {
    if !uniforms.night_lights_enabled || land_mask <= 0.0 {
        return Vector3::zero();
    }

    // El producto sin recortar distingue el lado nocturno del diurno.
    let light = (uniforms.light_position - fragment.world_position).normalized();
    let night = 1.0 - smoothstep(-0.20, 0.0, fragment.normal.dot(light));
    if night <= 0.0 {
        return Vector3::zero();
    }

    let lowlands = 1.0 - smoothstep(0.58, 0.65, height);
    let mask = land_mask * lowlands * cities * night;

    // Se suma emisión sin multiplicarla por la iluminación difusa.
    Vector3::new(1.0, 0.64, 0.19) * (mask * LIGHT_STRENGTH)
}
