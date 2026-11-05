use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
};
use raylib::prelude::*;

const CONTINENT_FREQUENCY: f32 = 2.8;
const SEA_LEVEL: f32 = 0.49;
const COAST_WIDTH: f32 = 0.025;
const SNOW_HEIGHT: f32 = 0.67;
const WATER_SHININESS: f32 = 72.0;
const WATER_SPECULAR: f32 = 0.85;

fn height_at(position: Vector3, uniforms: &Uniforms) -> f32 {
    let warp_position = position * 2.0;
    let warp = Vector3::new(
        sample(&uniforms.noise, warp_position),
        sample(&uniforms.noise, warp_position + Vector3::new(9.0, 3.0, 7.0)),
        sample(
            &uniforms.noise,
            warp_position + Vector3::new(4.0, 17.0, 2.0),
        ),
    );
    fbm(
        &uniforms.noise,
        (position + warp * 0.16) * CONTINENT_FREQUENCY,
    )
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let height = height_at(p, uniforms);
    let land_mask = smoothstep(SEA_LEVEL - 0.006, SEA_LEVEL + 0.006, height);

    let shallow = smoothstep(SEA_LEVEL - 0.10, SEA_LEVEL, height);
    let ocean = mix(
        Vector3::new(0.015, 0.12, 0.65),
        Vector3::new(0.03, 0.72, 0.86),
        shallow,
    );

    let latitude = p.normalized().y.abs();
    let moisture = fbm(&uniforms.noise, p * 4.5 + Vector3::new(21.0, 6.0, 4.0));
    let dry = (1.0 - smoothstep(0.38, 0.57, moisture)) * (1.0 - smoothstep(0.35, 0.7, latitude));
    let vegetation = mix(
        Vector3::new(0.12, 0.55, 0.11),
        Vector3::new(0.43, 0.82, 0.12),
        moisture,
    );
    let terrain = mix(vegetation, Vector3::new(0.88, 0.66, 0.27), dry);
    let beach = Vector3::new(1.0, 0.87, 0.52);
    let terrain = mix(
        beach,
        terrain,
        smoothstep(SEA_LEVEL, SEA_LEVEL + COAST_WIDTH, height),
    );

    let mountains = smoothstep(0.60, SNOW_HEIGHT, height);
    let terrain = mix(terrain, Vector3::new(0.44, 0.39, 0.28), mountains);
    let snow =
        smoothstep(SNOW_HEIGHT, SNOW_HEIGHT + 0.055, height).max(smoothstep(0.80, 0.96, latitude));
    let terrain = mix(terrain, Vector3::new(0.91, 0.97, 1.0), snow);
    let color = mix(ocean, terrain, land_mask);

    let diffuse = color * (0.16 + 0.84 * fragment.intensity);

    // El reflejo depende de la luz y de la posición de la cámara.
    let light = (uniforms.light_position - fragment.world_position).normalized();
    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let halfway = (light + view).normalized();
    let specular = if uniforms.specular_enabled && fragment.normal.dot(light) > 0.0 {
        fragment.normal.dot(halfway).max(0.0).powf(WATER_SHININESS)
    } else {
        0.0
    };

    // La máscara limita el reflejo al agua y mantiene la tierra mate.
    let water_mask = 1.0 - land_mask;
    let surface =
        diffuse + Vector3::new(0.83, 0.94, 1.0) * (specular * WATER_SPECULAR * water_mask);
    let surface = surface + crate::night::emission(fragment, uniforms, land_mask, height);
    crate::clouds::blend(surface, fragment, uniforms)
}
