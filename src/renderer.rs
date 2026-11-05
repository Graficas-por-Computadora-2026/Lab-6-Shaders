use crate::{
    math::*,
    shaders::{self, Params},
};
pub struct Mesh {
    pub vertices: Vec<V>,
    pub faces: Vec<[usize; 3]>,
}
impl Mesh {
    pub fn load() -> Self {
        let mut vertices = vec![];
        let mut faces = vec![];
        for line in include_str!("sphere.obj").lines() {
            let mut s = line.split_whitespace();
            match s.next() {
                Some("v") => {
                    let v: Vec<f32> = s.map(|x| x.parse().unwrap()).collect();
                    vertices.push(V(v[0], v[1], v[2]));
                }
                Some("f") => {
                    let f: Vec<usize> = s
                        .map(|x| x.split('/').next().unwrap().parse::<usize>().unwrap() - 1)
                        .collect();
                    for i in 1..f.len() - 1 {
                        faces.push([f[0], f[i], f[i + 1]]);
                    }
                }
                _ => {}
            }
        }
        Self { vertices, faces }
    }
}
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub rgb: Vec<u8>,
    depth: Vec<f32>,
    samples: Vec<Option<(V, V)>>,
    camera: Option<(u32, usize)>,
    workers: usize,
}
impl Frame {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            rgb: vec![0; w * h * 3],
            depth: vec![f32::NEG_INFINITY; w * h],
            samples: vec![None; w * h],
            camera: None,
            workers: std::thread::available_parallelism()
                .map_or(1, |n| n.get())
                .min(8),
        }
    }
    pub fn render(&mut self, mesh: &Mesh, kind: usize, u: &Params, yaw: f32) {
        let key = (yaw.to_bits(), mesh as *const Mesh as usize);
        if self.camera != Some(key) {
            self.rasterize(mesh, yaw);
            self.camera = Some(key);
        }
        let view = V(yaw.sin(), 0., yaw.cos());
        // Franjas disjuntas: cada hilo escribe exclusivamente sus píxeles RGB.
        let chunk = self.samples.len().div_ceil(self.workers);
        std::thread::scope(|scope| {
            for (pixels, samples) in self
                .rgb
                .chunks_mut(chunk * 3)
                .zip(self.samples.chunks(chunk))
            {
                scope.spawn(move || {
                    for (rgb, sample) in pixels.as_chunks_mut::<3>().0.iter_mut().zip(samples) {
                        if let Some((p, n)) = sample {
                            let color = shaders::shade(kind, *p, *n, view, u);
                            for (dst, value) in rgb.iter_mut().zip([color.0, color.1, color.2]) {
                                *dst = (value.clamp(0., 1.).sqrt() * 255.) as u8;
                            }
                        }
                    }
                });
            }
        });
    }
    fn rasterize(&mut self, mesh: &Mesh, yaw: f32) {
        self.samples.fill(None);
        self.depth.fill(f32::NEG_INFINITY);
        for y in 0..self.h {
            for x in 0..self.w {
                let i = (y * self.w + x) * 3;
                let star = ((x * 1973 + y * 9277 + 89173) % 104729) < 65;
                self.rgb[i..i + 3].copy_from_slice(if star {
                    &[95, 105, 130]
                } else {
                    &[7, 10, 22]
                });
            }
        }
        let right = V(yaw.cos(), 0., -yaw.sin());
        let up = V(0., 1., 0.);
        let view = V(yaw.sin(), 0., yaw.cos());
        let scale = self.h as f32 * 0.48;
        let project = |p: V| {
            V(
                self.w as f32 * 0.5 + p.dot(right) * scale,
                self.h as f32 * 0.5 - p.dot(up) * scale,
                p.dot(view),
            )
        };
        let projected: Vec<V> = mesh.vertices.iter().map(|&p| project(p)).collect();
        for &[ia, ib, ic] in &mesh.faces {
            let [a, b, c] = [projected[ia], projected[ib], projected[ic]];
            let area = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
            if area.abs() < 1e-7 {
                continue;
            }
            let xmin = a.0.min(b.0).min(c.0).floor().max(0.) as usize;
            let xmax = a.0.max(b.0).max(c.0).ceil().min(self.w as f32 - 1.) as usize;
            let ymin = a.1.min(b.1).min(c.1).floor().max(0.) as usize;
            let ymax = a.1.max(b.1).max(c.1).ceil().min(self.h as f32 - 1.) as usize;
            for y in ymin..=ymax {
                for x in xmin..=xmax {
                    let px = x as f32 + 0.5;
                    let py = y as f32 + 0.5;
                    let wb = ((px - a.0) * (c.1 - a.1) - (py - a.1) * (c.0 - a.0)) / area;
                    let wc = ((b.0 - a.0) * (py - a.1) - (b.1 - a.1) * (px - a.0)) / area;
                    let wa = 1. - wb - wc;
                    if wa < 0. || wb < 0. || wc < 0. {
                        continue;
                    }
                    let z = a.2 * wa + b.2 * wb + c.2 * wc;
                    let idx = y * self.w + x;
                    if z <= self.depth[idx] {
                        continue;
                    }
                    self.depth[idx] = z;
                    let p =
                        mesh.vertices[ia] * wa + mesh.vertices[ib] * wb + mesh.vertices[ic] * wc;
                    // Resolver visibilidad antes de ejecutar cualquier shader costoso.
                    self.samples[idx] = Some((p, p.unit()));
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parallel_shading_matches_single_worker_and_camera_cache_invalidates() {
        let mesh = Mesh::load();
        let mut serial = Frame::new(96, 96);
        serial.workers = 1;
        let mut parallel = Frame::new(96, 96);
        parallel.workers = 4;
        for kind in 0..3 {
            for yaw in [0., 0.4, 0.4, 0.] {
                let u = Params {
                    time: 2.,
                    ..Params::default()
                };
                // Forzar referencia sin reutilizar la rasterización.
                serial.camera = None;
                serial.render(&mesh, kind, &u, yaw);
                parallel.render(&mesh, kind, &u, yaw);
                assert_eq!(serial.rgb, parallel.rgb);
            }
        }
    }
    #[test]
    fn provided_mesh_and_shaders_render() {
        let mesh = Mesh::load();
        assert!(mesh.vertices.len() > 90);
        assert!(
            mesh.faces
                .iter()
                .flatten()
                .all(|&i| i < mesh.vertices.len())
        );
        let mut f = Frame::new(96, 96);
        let mut hashes = vec![];
        for k in 0..3 {
            f.render(&mesh, k, &Params::default(), 0.);
            assert!(f.depth.iter().filter(|z| z.is_finite()).count() > 1500);
            hashes.push(f.rgb.clone());
        }
        assert_ne!(hashes[0], hashes[1]);
        assert_ne!(hashes[1], hashes[2]);
        let first = hashes[0].clone();
        f.render(
            &mesh,
            0,
            &Params {
                time: 3.,
                ..Params::default()
            },
            0.,
        );
        assert_ne!(first, f.rgb);
    }
}
