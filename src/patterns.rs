use noise::{NoiseFn, OpenSimplex};
use raylib::prelude::*;

pub fn sample(noise: &OpenSimplex, position: Vector3) -> f32 {
    noise.get([position.x as f64, position.y as f64, position.z as f64]) as f32
}

pub fn fbm(noise: &OpenSimplex, mut position: Vector3) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut weight = 0.0;
    for _ in 0..3 {
        value += sample(noise, position) * amplitude;
        weight += amplitude;
        position = position * 2.0;
        amplitude *= 0.5;
    }
    value / weight * 0.5 + 0.5
}

pub fn mix(a: Vector3, b: Vector3, amount: f32) -> Vector3 {
    a * (1.0 - amount) + b * amount
}

pub fn smoothstep(low: f32, high: f32, value: f32) -> f32 {
    let t = ((value - low) / (high - low)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
