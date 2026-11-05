use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
};
use raylib::prelude::*;

const CLOUD_SPEED: f32 = 0.07;
const CLOUD_FREQUENCY: f32 = 4.0;
const CLOUD_OPACITY: f32 = 0.85;
static CLOUDS: std::sync::OnceLock<crate::surface::Surface<2>> = std::sync::OnceLock::new();
pub fn prepare(uniforms: &Uniforms) {
    CLOUDS.get_or_init(|| {
        crate::surface::Surface::new(|p| {
            let p = p * 0.8660254;
            let offset = Vector3::new(8.0, 15.0, 3.0);
            let curl = sample(&uniforms.noise, p * 5.0 + offset);
            let warped = p + Vector3::new(curl * 0.12, curl * 0.035, -curl * 0.09);
            let broad = fbm(&uniforms.noise, warped * CLOUD_FREQUENCY + offset);
            let detail = fbm(&uniforms.noise, warped * 27.0 + offset);
            [broad * 0.72 + detail * 0.28, detail]
        })
    });
}

pub fn blend(surface_color: Vector3, fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    if !uniforms.clouds_enabled || uniforms.cloud_coverage <= 0.0 {
        return surface_color;
    }

    // Solo cambian las coordenadas de las nubes. El terreno conserva las suyas.
    let p = fragment.object_position;
    let (sin, cos) = (uniforms.time * CLOUD_SPEED).sin_cos();
    let position = Vector3::new(p.x * cos - p.z * sin, p.y, p.x * sin + p.z * cos);
    let [density, detail] = CLOUDS.get().expect("Nubes preparadas").sample(position);

    // Al aumentar la cobertura, baja el umbral de la máscara.
    let threshold = 0.78 - uniforms.cloud_coverage * 0.56;
    let mask = if uniforms.cloud_coverage >= 1.0 {
        1.0
    } else {
        smoothstep(threshold, threshold + 0.12, density)
    };
    let cloud_color = Vector3::new(0.95, 0.98, 1.0)
        * ((0.86 + detail * 0.14) * (0.20 + 0.80 * fragment.intensity));
    mix(
        surface_color,
        cloud_color,
        mask * CLOUD_OPACITY * (0.65 + detail * 0.35),
    )
}
