// Shaders y funciones de interpolación.
pub mod console_styles;
pub mod noise_helpers;
pub mod planets;
pub mod stars;

use raylib::prelude::*;

// lerp
pub fn mix(a: Vector3, b: Vector3, amount: f32) -> Vector3 {
    a * (1.0 - amount) + b * amount
}

pub fn smoothstep(start: f32, end: f32, value: f32) -> f32 {
    let t = ((value - start) / (end - start)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

use crate::{Uniforms, fragment::Fragment};
pub const NAMES: [&str; 3] = [
    "IAN / HEARTFIRE",
    "JIWOO / TIDAL BLOOM",
    "RUKA / MONSTER STORM",
];
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    match uniforms.shader {
        0 => stars::sun(fragment, uniforms),
        1 => planets::rocky(fragment, uniforms),
        _ => planets::gas_giant(fragment, uniforms),
    }
}

use crate::vertex::Vertex;
// Transformar las posiciones del modelo a coordenadas de pantalla.
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let mut result = *vertex;
    result.world_position = vertex.object_position.transform_with(uniforms.model_matrix);
    let camera = result.world_position.transform_with(uniforms.view_matrix);
    result.transformed_position = camera.transform_with(uniforms.viewport_matrix);
    result
}
