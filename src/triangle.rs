use crate::{Uniforms, fragment::Fragment, vertex::Vertex};
use raylib::prelude::*;
// Generar los fragmentos de cada triángulo mediante un iterador.
pub fn triangle<'a>(
    vertices: &'a [Vertex],
    uniforms: &Uniforms,
) -> impl Iterator<Item = Fragment> + 'a {
    let light = uniforms.light;
    let [a, b, c] = [
        vertices[0].transformed_position,
        vertices[1].transformed_position,
        vertices[2].transformed_position,
    ];
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    let xmin = a.x.min(b.x).min(c.x).floor().max(0.) as usize;
    let xmax = a.x.max(b.x).max(c.x).ceil().min(uniforms.width as f32 - 1.) as usize;
    let ymin = a.y.min(b.y).min(c.y).floor().max(0.) as usize;
    let ymax =
        a.y.max(b.y)
            .max(c.y)
            .ceil()
            .min(uniforms.height as f32 - 1.) as usize;
    (ymin..=ymax)
        .flat_map(move |y| (xmin..=xmax).map(move |x| (x, y)))
        .filter_map(move |(x, y)| {
            if area.abs() < 1e-7 {
                return None;
            }
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let wb = ((px - a.x) * (c.y - a.y) - (py - a.y) * (c.x - a.x)) / area;
            let wc = ((b.x - a.x) * (py - a.y) - (b.y - a.y) * (px - a.x)) / area;
            let wa = 1. - wb - wc;
            if wa < 0. || wb < 0. || wc < 0. {
                return None;
            }
            let p = vertices[0].object_position * wa
                + vertices[1].object_position * wb
                + vertices[2].object_position * wc;
            let normal =
                (vertices[0].normal * wa + vertices[1].normal * wb + vertices[2].normal * wc)
                    .normalized();
            Some(Fragment {
                position: Vector2::new(x as f32, y as f32),
                depth: a.z * wa + b.z * wb + c.z * wc,
                object_position: p,
                normal,
                intensity: normal.dot(light).max(0.),
            })
        })
}
