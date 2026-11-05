use std::ops::{Add, Div, Mul, Neg, Sub};
#[derive(Clone, Copy, Debug, Default)]
pub struct V(pub f32, pub f32, pub f32);
impl Add for V {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        V(self.0 + b.0, self.1 + b.1, self.2 + b.2)
    }
}
impl Sub for V {
    type Output = Self;
    fn sub(self, b: Self) -> Self {
        V(self.0 - b.0, self.1 - b.1, self.2 - b.2)
    }
}
impl Mul<f32> for V {
    type Output = Self;
    fn mul(self, b: f32) -> Self {
        V(self.0 * b, self.1 * b, self.2 * b)
    }
}
impl Div<f32> for V {
    type Output = Self;
    fn div(self, b: f32) -> Self {
        self * (1. / b)
    }
}
impl Neg for V {
    type Output = Self;
    fn neg(self) -> Self {
        self * (-1.)
    }
}
impl V {
    pub fn dot(self, b: Self) -> f32 {
        self.0 * b.0 + self.1 * b.1 + self.2 * b.2
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn unit(self) -> Self {
        self / self.len().max(1e-8)
    }
}
pub fn mix(a: V, b: V, t: f32) -> V {
    a * (1. - t) + b * t
}
pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(374761393)
        ^ (y as u32).wrapping_mul(668265263)
        ^ (z as u32).wrapping_mul(2147483647);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}
pub fn noise(p: V) -> f32 {
    let V(x, y, z) = p;
    let (i, j, k) = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
    let (u, v, w) = (
        smooth(0., 1., x - x.floor()),
        smooth(0., 1., y - y.floor()),
        smooth(0., 1., z - z.floor()),
    );
    let mut n = 0.;
    for a in 0..2 {
        for b in 0..2 {
            for c in 0..2 {
                n += hash(i + a, j + b, k + c)
                    * if a == 0 { 1. - u } else { u }
                    * if b == 0 { 1. - v } else { v }
                    * if c == 0 { 1. - w } else { w };
            }
        }
    }
    n
}
pub fn fbm(mut p: V) -> f32 {
    let mut n = 0.;
    let mut a = 0.55;
    for _ in 0..4 {
        n += a * noise(p);
        p = p * 2.03 + V(7.1, 3.7, 1.9);
        a *= 0.48;
    }
    n
}
