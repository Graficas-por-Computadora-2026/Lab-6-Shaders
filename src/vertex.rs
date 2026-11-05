use raylib::prelude::*;
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub object_position: Vector3,
    pub world_position: Vector3,
    pub transformed_position: Vector3,
    pub normal: Vector3,
}
impl Vertex {
    pub fn new(position: Vector3) -> Self {
        Self {
            object_position: position,
            world_position: position,
            transformed_position: position,
            normal: position.normalized(),
        }
    }
}
