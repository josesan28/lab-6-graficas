use crate::{
    Uniforms,
    fragment::Fragment,
    patterns::{fbm, mix, sample, smoothstep},
    surface::Surface,
};
use raylib::prelude::*;
use std::sync::OnceLock;
static ICE: OnceLock<Surface<3>> = OnceLock::new();

pub fn prepare(u: &Uniforms) {
    ICE.get_or_init(|| {
        Surface::new(|p| {
            let warp = sample(&u.noise, p * 4.0);
            let fractures = sample(&u.noise, p * 15.0 + Vector3::new(warp * 0.5, 2.0, 0.0)).abs();
            [
                fbm(&u.noise, p * 5.0),
                1.0 - smoothstep(0.015, 0.065, fractures),
                fbm(&u.noise, p * 65.0),
            ]
        })
    });
}

pub fn shade(f: &Fragment, u: &Uniforms) -> Vector3 {
    let p = f.object_position.normalized();
    let [height, cracks, grain] = ICE.get().expect("Hielo preparado").sample(p);
    let snow = smoothstep(0.42, 0.58, height);
    let lake = mix(
        Vector3::new(0.08, 0.24, 0.60),
        Vector3::new(0.20, 0.68, 0.86),
        height,
    );
    let frost = mix(
        Vector3::new(0.58, 0.48, 0.82),
        Vector3::new(0.87, 0.96, 1.0),
        grain,
    );
    let albedo = mix(lake, frost, snow);
    let shadow = if u.cloud_shadows_enabled {
        crate::clouds::shadow(f, u)
    } else {
        1.0
    };
    let surface = albedo * (0.19 + 0.81 * f.intensity * shadow);
    let view = (u.camera_position - f.world_position).normalized();
    let light = (u.light_position - f.world_position).normalized();
    let specular = if u.specular_enabled {
        f.normal.dot((view + light).normalized()).max(0.0).powi(64) * f.intensity * shadow
    } else {
        0.0
    };
    let glints = smoothstep(0.64, 0.74, grain) * specular * 1.7;
    let surface = surface
        + Vector3::new(0.42, 0.94, 1.0) * (cracks * 0.22 + glints)
        + Vector3::new(0.75, 0.89, 1.0) * specular * 0.7;
    let surface = crate::clouds::blend(surface, f, u);
    let aurora = if u.aurora_enabled {
        let curtain =
            (p.y * 26.0 + (p.x * 6.0 + p.z * 5.0 + u.time * 0.35).sin() * 1.8).sin() * 0.5 + 0.5;
        let polar = smoothstep(0.38, 0.66, p.y.abs()) * (1.0 - smoothstep(0.84, 0.98, p.y.abs()));
        let color = mix(
            Vector3::new(0.12, 0.95, 0.60),
            Vector3::new(0.68, 0.24, 1.0),
            curtain,
        );
        color * (polar * smoothstep(0.42, 0.95, curtain) * 0.48)
    } else {
        Vector3::zero()
    };
    let rim = (1.0 - f.normal.dot(view).max(0.0)).powi(4);
    surface + aurora + Vector3::new(0.19, 0.38, 0.65) * rim * 0.4
}
