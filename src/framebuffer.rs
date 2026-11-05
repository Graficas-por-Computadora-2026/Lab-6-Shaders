use crate::fragment::Fragment;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub(crate) struct GeometryKey {
    pub vertices: u64,
    pub model: Matrix,
    pub view: Matrix,
    pub viewport: Matrix,
}
pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub colors: Vec<Vector3>,
    pub depths: Vec<f32>,
    pub rgb: Vec<u8>,
    background: Vec<Vector3>,
    pub(crate) fragments: Vec<Option<Fragment>>,
    pub(crate) geometry: Option<GeometryKey>,
    pub(crate) workers: usize,
}
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let background = (0..width * height)
            .map(|i| {
                let (x, y) = (i % width, i / width);
                let bytes = if ((x * 1973 + y * 9277 + 89173) % 104729) < 65 {
                    [95., 105., 130.]
                } else {
                    [7., 10., 22.]
                };
                // La conversión final usa raíz cuadrada; invertir para conservar el fondo.
                Vector3::new(
                    (bytes[0] / 255_f32).powi(2),
                    (bytes[1] / 255_f32).powi(2),
                    (bytes[2] / 255_f32).powi(2),
                )
            })
            .collect::<Vec<_>>();
        Self {
            width,
            height,
            colors: background.clone(),
            depths: vec![f32::INFINITY; width * height],
            rgb: vec![0; width * height * 3],
            background,
            fragments: vec![None; width * height],
            geometry: None,
            workers: std::thread::available_parallelism()
                .map_or(1, |n| n.get())
                .min(8),
        }
    }
    pub fn clear(&mut self) {
        self.colors.copy_from_slice(&self.background);
        self.depths.fill(f32::INFINITY);
        self.fragments.fill(None);
        self.geometry = None;
    }
    pub fn update_rgb(&mut self) {
        for (rgb, color) in self.rgb.as_chunks_mut::<3>().0.iter_mut().zip(&self.colors) {
            for (dst, value) in rgb.iter_mut().zip([color.x, color.y, color.z]) {
                *dst = (value.clamp(0., 1.).sqrt() * 255.) as u8;
            }
        }
    }
}
