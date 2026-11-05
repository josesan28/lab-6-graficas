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

    let [mask, detail] = density_at(fragment.object_position, uniforms);
    let cloud_color = Vector3::new(0.95, 0.98, 1.0)
        * ((0.86 + detail * 0.14) * (0.20 + 0.80 * fragment.intensity));
    mix(
        surface_color,
        cloud_color,
        mask * CLOUD_OPACITY * (0.65 + detail * 0.35),
    )
}

fn density_at(p: Vector3, uniforms: &Uniforms) -> [f32; 2] {
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
    [mask, detail]
}

// Intersección analítica del rayo hacia la luz con la altura de las nubes.
// No añade mallas: consulta el mismo campo que se usa para dibujar las nubes.
pub fn shadow(fragment: &Fragment, uniforms: &Uniforms) -> f32 {
    if !uniforms.clouds_enabled || uniforms.cloud_coverage <= 0.0 {
        return 1.0;
    }
    let light = (uniforms.light_position - fragment.world_position).normalized();
    let light = crate::patterns::object_direction(light, uniforms);
    let p = fragment.object_position;
    if p.normalized().dot(light) <= 0.02 {
        return 1.0;
    }
    let [mask, _] = density_at(cloud_intersection(p, light), uniforms);
    1.0 - mask * 0.65
}

fn cloud_intersection(p: Vector3, light: Vector3) -> Vector3 {
    let b = p.dot(light);
    let distance = -b + (b * b + 0.93_f32.powi(2) - p.dot(p)).max(0.0).sqrt();
    p + light * distance
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shadow_projection_follows_light_on_cloud_shell() {
        let p = Vector3::new(0.0, 0.0, 0.8660254);
        let a = cloud_intersection(p, Vector3::new(0.8, 0.0, 0.6));
        let b = cloud_intersection(p, Vector3::new(-0.8, 0.0, 0.6));
        assert!((a.length() - 0.93).abs() < 0.00001);
        assert!((b.length() - 0.93).abs() < 0.00001);
        assert!(a.x > 0.0 && b.x < 0.0);
        assert!((a.x + b.x).abs() < 0.00001);
    }
}
