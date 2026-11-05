// Superficie emisiva con ruido y movimiento del material.
use super::{mix, noise_helpers::fbm, smoothstep};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;
pub fn sun(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let t = u.time;
    let warp = Vector3::new(
        fbm(&u.noise, p * 4. + Vector3::new(t * 0.1, 0., 0.)),
        fbm(&u.noise, p * 4. + Vector3::new(0., t * 0.12, 8.)),
        0.,
    );
    // Ondas y celdas animadas combinadas con turbulencia.
    let wave_warp = (p.x * 11.0 + t).sin() * (p.z * 13.0 - t * 0.7).sin();
    let wave = (p.y * 24.0 + wave_warp * 3.0 + t * 1.5).sin() * 0.5 + 0.5;
    let cells = (p.x * 31.0 + p.z * 17.0 + t).sin() * 0.5 + 0.5;
    let heat = (wave * 0.75 + cells * 0.25).powf(1.5);
    let turbulent = fbm(
        &u.noise,
        p * 11. + warp * 3. + Vector3::new(0., 0., t * 0.2),
    );
    let n = turbulent * 0.75 + heat * 0.25;
    let veins = (n * 24. + p.y * 9.).sin().abs();
    let spots = smoothstep(
        0.56,
        0.7,
        fbm(&u.noise, p * 5. + Vector3::new(t * 0.07, 9., 0.)),
    );
    let base = mix(
        Vector3::new(0.65, 0.025, 0.15),
        Vector3::new(1., 0.56, 0.24),
        smoothstep(0.25, 0.65, n),
    );
    let hot = mix(
        base,
        Vector3::new(1., 0.94, 0.7),
        smoothstep(0.62, 0.95, veins) * 0.75,
    );
    mix(hot, Vector3::new(0.19, 0.008, 0.065), spots * 0.87)
}
