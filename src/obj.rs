use crate::vertex::Vertex;
use raylib::prelude::*;
pub fn load_sphere() -> Vec<Vertex> {
    let mut positions = Vec::new();
    let mut vertices = Vec::new();
    for line in include_str!("sphere.obj").lines() {
        let mut words = line.split_whitespace();
        match words.next() {
            Some("v") => {
                let xyz: Vec<f32> = words
                    .map(|s| s.parse().expect("Coordenada OBJ inválida"))
                    .collect();
                positions.push(Vector3::new(xyz[0], xyz[1], xyz[2]));
            }
            Some("f") => {
                let face: Vec<usize> = words
                    .map(|s| s.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                    .collect();
                for i in 1..face.len() - 1 {
                    for index in [face[0], face[i], face[i + 1]] {
                        vertices.push(Vertex::new(positions[index]));
                    }
                }
            }
            _ => {}
        }
    }
    vertices
}
