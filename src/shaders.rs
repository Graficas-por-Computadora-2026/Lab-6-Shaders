use crate::math::*;
#[derive(Clone, Copy)]
pub struct Params {
    pub time: f32,
    pub light: V,
    pub clouds: bool,
    pub coverage: f32,
    pub bump: f32,
}
impl Default for Params {
    fn default() -> Self {
        Self {
            time: 0.,
            light: V(-0.65, 0.4, 0.65).unit(),
            clouds: true,
            coverage: 0.48,
            bump: 0.12,
        }
    }
}
pub const NAMES: [&str; 3] = [
    "IAN / HEARTFIRE",
    "JIWOO / TIDAL BLOOM",
    "RUKA / MONSTER STORM",
];
fn height(p: V) -> f32 {
    fbm(p * 3.3 + V(1.4, 0., 0.))
}
fn cloud(p: V, u: &Params) -> f32 {
    let t = u.time * 0.045;
    let q = V(
        p.0 * t.cos() + p.2 * t.sin(),
        p.1,
        -p.0 * t.sin() + p.2 * t.cos(),
    );
    smooth(u.coverage, u.coverage + 0.16, fbm(q * 6. + V(3., 1., 5.)))
}
pub fn star(p: V, u: &Params) -> V {
    let t = u.time;
    let warp = V(
        fbm(p * 4. + V(t * 0.1, 0., 0.)),
        fbm(p * 4. + V(0., t * 0.12, 8.)),
        0.,
    );
    let n = fbm(p * 11. + warp * 3. + V(0., 0., t * 0.2));
    let veins = (n * 24. + p.1 * 9.).sin().abs();
    let spots = smooth(0.56, 0.7, fbm(p * 5. + V(t * 0.07, 9., 0.)));
    let base = mix(
        V(0.65, 0.025, 0.15),
        V(1., 0.56, 0.24),
        smooth(0.25, 0.65, n),
    );
    let hot = mix(base, V(1., 0.94, 0.7), smooth(0.62, 0.95, veins) * 0.75);
    mix(hot, V(0.19, 0.008, 0.065), spots * 0.87)
}
pub fn rocky(p: V, n: V, view: V, u: &Params) -> V {
    let h = height(p);
    let land = smooth(0.44, 0.48, h);
    let e = 0.007;
    let grad = V(
        height(p + V(e, 0., 0.)) - height(p - V(e, 0., 0.)),
        height(p + V(0., e, 0.)) - height(p - V(0., e, 0.)),
        height(p + V(0., 0., e)) - height(p - V(0., 0., e)),
    ) / (2. * e);
    let bn = (n - (grad - n * n.dot(grad)) * u.bump * land).unit();
    let d = bn.dot(u.light).max(0.);
    let ocean = mix(
        V(0.015, 0.08, 0.19),
        V(0.02, 0.55, 0.61),
        smooth(0.3, 0.46, h),
    );
    let ground = mix(
        V(0.15, 0.32, 0.23),
        V(0.83, 0.49, 0.62),
        smooth(0.49, 0.68, h),
    );
    let mut base = mix(ocean, ground, land);
    base = mix(
        base,
        V(0.88, 0.96, 1.),
        smooth(0.72, 0.91, p.1.abs() + h * 0.13),
    );
    let half = (u.light + view).unit();
    let spec = bn.dot(half).max(0.).powf(95.) * (1. - land) * d;
    let mut color = base * (0.075 + 0.925 * d) + V(0.8, 0.94, 1.) * spec * 1.5;
    // Misma máscara muestreada hacia la luz en una capa a altura positiva.
    if u.clouds {
        let radial = n.dot(u.light);
        let shadow_p = (p + (u.light - n * radial) * (0.065 / radial.max(0.18))).unit() * p.len();
        let shadow = cloud(shadow_p, u) * smooth(0., 0.25, radial);
        color = color * (1. - 0.48 * shadow);
    }
    let city = smooth(0.72, 0.82, noise(p * 170.))
        * smooth(0.47, 0.52, h)
        * (1. - smooth(0., 0.22, n.dot(u.light)));
    color = color + V(1., 0.56, 0.22) * city * 0.8;
    if u.clouds {
        color = mix(
            color,
            V(0.84, 0.94, 1.) * (0.23 + 0.77 * n.dot(u.light).max(0.)),
            cloud(p, u) * 0.88,
        );
    }
    color + V(0.08, 0.4, 0.65) * (1. - n.dot(view).max(0.)).powf(4.) * 0.38
}
pub fn gas(p: V, n: V, view: V, u: &Params) -> V {
    let mut lon = p.0.atan2(p.2);
    let mut lat = (p.1 / p.len()).asin();
    let dx = (lon - 0.35 + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let dy = lat - 0.17;
    let r = ((dx / 0.5).powi(2) + (dy / 0.32).powi(2)).sqrt();
    let angle = 3.4 * (-r * r * 1.3).exp();
    lon = 0.35 + dx * angle.cos() - dy * angle.sin();
    lat = 0.17 + dx * angle.sin() + dy * angle.cos();
    let q = V(lon.cos() * lat.cos(), lat.sin(), lon.sin() * lat.cos());
    let flow = fbm(q * 8. + V(u.time * 0.07, 0., 0.));
    let bands = (lat * 32. + flow * 6. + (lon * 5. + u.time * 0.15).sin() * 0.65).sin() * 0.5 + 0.5;
    let mut c = mix(
        V(0.12, 0.035, 0.23),
        V(0.82, 0.25, 0.4),
        smooth(0.12, 0.58, bands),
    );
    c = mix(c, V(1., 0.77, 0.58), smooth(0.65, 0.92, bands));
    let filament = (lat * 110. + flow * 14.).sin() * 0.5 + 0.5;
    c = c * (0.8 + 0.2 * filament);
    c = mix(c, V(0.96, 0.51, 0.36), (-r * r * 3.).exp() * 0.3);
    c * (0.17 + 0.83 * n.dot(u.light).max(0.))
        + V(0.3, 0.13, 0.4) * (1. - n.dot(view).max(0.)).powf(4.) * 0.3
}
pub fn shade(kind: usize, p: V, n: V, view: V, u: &Params) -> V {
    match kind {
        0 => star(p, u),
        1 => rocky(p, n, view, u),
        _ => gas(p, n, view, u),
    }
}
