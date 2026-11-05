use noise::{NoiseFn, OpenSimplex};
use raylib::prelude::*;

pub fn sample(noise: &OpenSimplex, position: Vector3) -> f32 {
    noise.get([position.x as f64, position.y as f64, position.z as f64]) as f32
}

pub fn fbm(noise: &OpenSimplex, mut position: Vector3) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut weight = 0.0;
    for _ in 0..4 {
        value += sample(noise, position) * amplitude;
        weight += amplitude;
        position *= 2.0;
        amplitude *= 0.5;
    }
    value / weight * 0.5 + 0.5
}
