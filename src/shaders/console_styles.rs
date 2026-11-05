use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

// Iluminación suave parametrizada para cada material.
pub fn smooth(
    fragment: &Fragment,
    _uniforms: &Uniforms,
    color: Vector3,
    ambient: f32,
    diffuse: f32,
) -> Vector3 {
    color * (ambient + fragment.intensity * diffuse)
}
