use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, sample, smoothstep},
};
use raylib::prelude::*;

const CITY_FREQUENCY: f32 = 65.0;
const CITY_THRESHOLD: f32 = 0.32;
const LIGHT_STRENGTH: f32 = 1.4;

pub fn emission(fragment: &Fragment, uniforms: &Uniforms, land_mask: f32, height: f32) -> Vector3 {
    if !uniforms.night_lights_enabled || land_mask <= 0.0 {
        return Vector3::zero();
    }

    // El producto sin recortar distingue el lado nocturno del diurno.
    let light = (uniforms.light_position - fragment.world_position).normalized();
    let night = 1.0 - smoothstep(-0.20, 0.0, fragment.normal.dot(light));
    if night <= 0.0 {
        return Vector3::zero();
    }

    let p = fragment.object_position;
    // Un ruido agrupa las luces y otro determina los puntos dentro de cada grupo.
    let settlements = fbm(&uniforms.noise, p * 7.0 + Vector3::new(12.0, 4.0, 19.0));
    let clusters = smoothstep(0.46, 0.64, settlements);
    let points = sample(&uniforms.noise, p * CITY_FREQUENCY);
    let lights = smoothstep(CITY_THRESHOLD, CITY_THRESHOLD + 0.16, points);
    let lowlands = 1.0 - smoothstep(0.58, 0.65, height);
    let mask = land_mask * lowlands * clusters * lights * night;

    // Se suma emisión sin multiplicarla por la iluminación difusa.
    Vector3::new(1.0, 0.64, 0.19) * (mask * LIGHT_STRENGTH)
}
