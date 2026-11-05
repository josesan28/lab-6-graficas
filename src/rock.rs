use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
};
use raylib::prelude::*;

const CONTINENT_FREQUENCY: f32 = 2.8;
const SEA_LEVEL: f32 = 0.49;
const COAST_WIDTH: f32 = 0.005;
const SNOW_HEIGHT: f32 = 0.67;
const WATER_SHININESS: f32 = 72.0;
const WATER_SPECULAR: f32 = 0.85;
const BUMP_STRENGTH: f32 = 0.018;
const BUMP_STEP: f32 = 0.003;

static TERRAIN: std::sync::OnceLock<crate::surface::Surface<8>> = std::sync::OnceLock::new();

pub fn prepare(uniforms: &Uniforms) {
    TERRAIN.get_or_init(|| {
        crate::surface::Surface::new(|direction| {
            let p = direction * 0.8660254;
            let height = height_at(p, uniforms);
            let moisture = fbm(&uniforms.noise, p * 4.5 + Vector3::new(21.0, 6.0, 4.0));
            let relief = |p: Vector3| {
                let ridge = 1.0 - sample(&uniforms.noise, p * 23.0).abs();
                sample(&uniforms.noise, p * 9.0) * 0.18 + ridge.powi(4) * 0.45
            };
            let center = relief(p);
            let gradient = Vector3::new(
                relief(p + Vector3::new(BUMP_STEP, 0.0, 0.0)) - center,
                relief(p + Vector3::new(0.0, BUMP_STEP, 0.0)) - center,
                relief(p + Vector3::new(0.0, 0.0, BUMP_STEP)) - center,
            ) / BUMP_STEP;
            let grain = fbm(&uniforms.noise, p * 48.0);
            let settlements = fbm(&uniforms.noise, p * 7.0 + Vector3::new(12.0, 4.0, 19.0));
            let cities = smoothstep(0.46, 0.64, settlements)
                * smoothstep(0.32, 0.48, sample(&uniforms.noise, p * 65.0));
            [
                height, moisture, gradient.x, gradient.y, gradient.z, grain, center, cities,
            ]
        })
    });
}

fn bumped_normal(
    fragment: &Fragment,
    uniforms: &Uniforms,
    height: f32,
    gradient: Vector3,
) -> Vector3 {
    let land = smoothstep(SEA_LEVEL + 0.012, SEA_LEVEL + 0.065, height);
    if !uniforms.bump_enabled || land <= 0.0 {
        return fragment.normal;
    }
    let transformed = crate::pipeline::transform(
        &uniforms.model_matrix,
        Vector4::new(gradient.x, gradient.y, gradient.z, 0.0),
    );
    let gradient = Vector3::new(transformed.x, transformed.y, transformed.z);
    let normal = fragment.normal;
    (normal - (gradient - normal * gradient.dot(normal)) * (BUMP_STRENGTH * land)).normalized()
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
    let base = fbm(
        &uniforms.noise,
        (position + warp * 0.16) * CONTINENT_FREQUENCY,
    );
    // Octavas finas para cabos, ensenadas e islas; solo se calculan al iniciar.
    base + sample(&uniforms.noise, position * 24.0) * 0.018
        + sample(&uniforms.noise, position * 57.0) * 0.008
        + sample(&uniforms.noise, position * 110.0) * 0.003
}

pub fn shade(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let data = TERRAIN.get().expect("Terreno preparado").sample(p);
    let height = data[0];
    let land_mask = smoothstep(SEA_LEVEL - 0.002, SEA_LEVEL + 0.002, height);

    let shallow = smoothstep(SEA_LEVEL - 0.035, SEA_LEVEL - 0.004, height);
    let ocean = mix(
        Vector3::new(0.012, 0.075, 0.32),
        Vector3::new(0.025, 0.48, 0.62),
        shallow,
    );

    // El océano no necesita los cálculos de vegetación, roca y nieve.
    let terrain = if land_mask > 0.0 {
        let latitude = p.normalized().y.abs();
        let moisture = data[1];
        let dry =
            (1.0 - smoothstep(0.43, 0.48, moisture)) * (1.0 - smoothstep(0.35, 0.7, latitude));
        let vegetation = mix(
            Vector3::new(0.055, 0.23, 0.11),
            Vector3::new(0.28, 0.48, 0.17),
            smoothstep(0.48, 0.53, moisture),
        );
        let terrain = mix(vegetation, Vector3::new(0.88, 0.66, 0.27), dry);
        let beach = Vector3::new(0.76, 0.72, 0.48);
        let terrain = mix(
            beach,
            terrain,
            smoothstep(SEA_LEVEL, SEA_LEVEL + COAST_WIDTH, height),
        );

        let mountains = smoothstep(0.55, 0.65, height + data[6] * 0.045);
        let terrain = mix(terrain, Vector3::new(0.44, 0.39, 0.28), mountains);
        let snow = smoothstep(SNOW_HEIGHT, SNOW_HEIGHT + 0.025, height)
            .max(smoothstep(0.80, 0.96, latitude));
        let terrain = mix(terrain, Vector3::new(0.91, 0.97, 1.0), snow);
        let grain = data[5];
        terrain * (0.72 + grain * 0.38)
    } else {
        Vector3::zero()
    };
    let color = mix(ocean, terrain, land_mask);

    let normal = bumped_normal(
        fragment,
        uniforms,
        height,
        Vector3::new(data[2], data[3], data[4]),
    );
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
    let surface = surface + crate::night::emission(fragment, uniforms, land_mask, height, data[7]);
    let surface = crate::clouds::blend(surface, fragment, uniforms);
    let rim = (1.0 - fragment.normal.dot(view).max(0.0)).powi(4);
    mix(
        surface,
        Vector3::new(0.16, 0.43, 0.78),
        rim * (0.12 + 0.28 * intensity),
    )
}
