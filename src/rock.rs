use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
};
use raylib::prelude::*;

const CONTINENT_FREQUENCY: f32 = 2.8;
const SEA_LEVEL: f32 = 0.49;
const COAST_WIDTH: f32 = 0.014;
const SNOW_HEIGHT: f32 = 0.67;
const WATER_SHININESS: f32 = 72.0;
const WATER_SPECULAR: f32 = 0.85;
const BUMP_STRENGTH: f32 = 0.035;
const BUMP_STEP: f32 = 0.003;

fn relief_at(position: Vector3, uniforms: &Uniforms) -> f32 {
    sample(&uniforms.noise, position * 9.0)
}

fn bumped_normal(fragment: &Fragment, uniforms: &Uniforms, height: f32) -> Vector3 {
    let land = smoothstep(SEA_LEVEL + 0.012, SEA_LEVEL + 0.065, height);
    if !uniforms.bump_enabled || land <= 0.0 {
        return fragment.normal;
    }

    // Reutiliza la altura central para calcular las tres pendientes.
    let p = fragment.object_position;
    let center = relief_at(p, uniforms);
    let x = Vector3::new(BUMP_STEP, 0.0, 0.0);
    let y = Vector3::new(0.0, BUMP_STEP, 0.0);
    let z = Vector3::new(0.0, 0.0, BUMP_STEP);
    let gradient = Vector3::new(
        relief_at(p + x, uniforms) - center,
        relief_at(p + y, uniforms) - center,
        relief_at(p + z, uniforms) - center,
    ) / BUMP_STEP;

    // Lleva la pendiente al espacio de iluminación y elimina su componente normal.
    let transformed = crate::pipeline::transform(
        &uniforms.model_matrix,
        Vector4::new(gradient.x, gradient.y, gradient.z, 0.0),
    );
    let gradient = Vector3::new(transformed.x, transformed.y, transformed.z);
    let normal = fragment.normal;
    let tangent_gradient = gradient - normal * gradient.dot(normal);
    (normal - tangent_gradient * (BUMP_STRENGTH * land)).normalized()
}

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
    let land_mask = smoothstep(SEA_LEVEL - 0.002, SEA_LEVEL + 0.002, height);

    let shallow = smoothstep(SEA_LEVEL - 0.035, SEA_LEVEL - 0.004, height);
    let ocean = mix(
        Vector3::new(0.015, 0.12, 0.65),
        Vector3::new(0.03, 0.72, 0.86),
        shallow,
    );

    // El océano no necesita los cálculos de vegetación, roca y nieve.
    let terrain = if land_mask > 0.0 {
        let latitude = p.normalized().y.abs();
        let moisture = fbm(&uniforms.noise, p * 4.5 + Vector3::new(21.0, 6.0, 4.0));
        let dry =
            (1.0 - smoothstep(0.43, 0.48, moisture)) * (1.0 - smoothstep(0.35, 0.7, latitude));
        let vegetation = mix(
            Vector3::new(0.07, 0.40, 0.10),
            Vector3::new(0.40, 0.78, 0.10),
            smoothstep(0.48, 0.53, moisture),
        );
        let terrain = mix(vegetation, Vector3::new(0.88, 0.66, 0.27), dry);
        let beach = Vector3::new(1.0, 0.87, 0.52);
        let terrain = mix(
            beach,
            terrain,
            smoothstep(SEA_LEVEL, SEA_LEVEL + COAST_WIDTH, height),
        );

        let mountains = smoothstep(0.60, 0.63, height);
        let terrain = mix(terrain, Vector3::new(0.44, 0.39, 0.28), mountains);
        let snow = smoothstep(SNOW_HEIGHT, SNOW_HEIGHT + 0.025, height)
            .max(smoothstep(0.80, 0.96, latitude));
        let terrain = mix(terrain, Vector3::new(0.91, 0.97, 1.0), snow);
        let grain = sample(&uniforms.noise, p * 38.0) * 0.5 + 0.5;
        terrain * (0.84 + grain * 0.16)
    } else {
        Vector3::zero()
    };
    let color = mix(ocean, terrain, land_mask);

    let normal = bumped_normal(fragment, uniforms, height);
    let light = (uniforms.light_position - fragment.world_position).normalized();
    let intensity = normal.dot(light).max(0.0);
    let diffuse = color * (0.16 + 0.84 * intensity);

    // El reflejo depende de la luz y de la posición de la cámara.
    let view = (uniforms.camera_position - fragment.world_position).normalized();
    let halfway = (light + view).normalized();
    let specular = if uniforms.specular_enabled && intensity > 0.0 {
        normal.dot(halfway).max(0.0).powf(WATER_SHININESS)
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
