use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{mix, sample, smoothstep},
};
use raylib::prelude::*;

const BAND_FREQUENCY: f32 = 22.0;
const DISTORTION: f32 = 2.6;
const FLOW_SPEED: f32 = 0.10;
const DETAIL_STRENGTH: f32 = 0.15;
const VORTEX_RADIUS: f32 = 0.48;
const VORTEX_STRENGTH: f32 = 4.5;

fn vortex(position: Vector3) -> Vector3 {
    let center = Vector3::new(-0.30, -0.12, 1.0).normalized();
    let alignment = position.normalized().dot(center).clamp(-1.0, 1.0);
    if alignment <= VORTEX_RADIUS.cos() {
        return position;
    }
    let distance = alignment.acos();

    // El giro disminuye con la distancia hasta llegar a cero en el borde.
    let influence = 1.0 - smoothstep(0.0, VORTEX_RADIUS, distance);
    let angle = VORTEX_STRENGTH * influence;
    let (sin, cos) = angle.sin_cos();

    // Rota las coordenadas alrededor del eje del vórtice.
    position * cos + center.cross(position) * sin + center * (center.dot(position) * (1.0 - cos))
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = if uniforms.vortex_enabled {
        vortex(fragment.object_position)
    } else {
        fragment.object_position
    };

    // Cada latitud desplaza el patrón a una velocidad diferente.
    let angle = uniforms.time * FLOW_SPEED * (1.0 + 0.35 * (p.y * 6.0).sin());
    let (sin, cos) = angle.sin_cos();
    let surface = Vector3::new(p.x * cos - p.z * sin, p.y, p.x * sin + p.z * cos);

    // El ruido y la onda cambian la posición de las bandas.
    let turbulence = sample(&uniforms.noise, surface * 4.0);
    let wave = (surface.x * 6.0 + surface.z * 4.0).sin();
    let phase = surface.y * BAND_FREQUENCY + turbulence * DISTORTION + wave * 0.5;
    let bands = phase.sin() * 0.5 + 0.5;

    let purple = Vector3::new(0.32, 0.12, 0.62);
    let lavender = Vector3::new(0.76, 0.55, 0.96);
    let cream = Vector3::new(0.98, 0.90, 0.73);
    let color = mix(purple, lavender, smoothstep(0.28, 0.56, bands));
    let color = mix(color, cream, smoothstep(0.76, 0.88, bands));

    // Las líneas secundarias siguen la misma deformación que las bandas.
    let detail = sample(&uniforms.noise, surface * 18.0) * 0.5 + 0.5;
    let lines = (phase * 3.0 + detail * 3.0).sin() * 0.5 + 0.5;
    let line_mask = smoothstep(0.76, 0.95, lines) * (1.0 - smoothstep(0.65, 0.90, bands));
    let color = mix(color, Vector3::new(0.20, 0.78, 0.84), line_mask * 0.55);
    let fine_lines = (phase * 6.0 + detail * 5.0).sin() * 0.5 + 0.5;
    let color = mix(
        color,
        Vector3::new(0.86, 0.78, 0.98),
        smoothstep(0.85, 0.98, fine_lines) * 0.22,
    );
    let color = color * (1.0 - DETAIL_STRENGTH + detail * DETAIL_STRENGTH);

    color * (0.16 + 0.84 * fragment.intensity)
}
