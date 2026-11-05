// Superficies procedurales del planeta rocoso y el gigante gaseoso.
use super::{
    console_styles, mix,
    noise_helpers::{fbm, sample},
    smoothstep,
};
use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;
fn height(p: Vector3, u: &Uniforms) -> f32 {
    fbm(&u.noise, p * 3.3 + Vector3::new(1.4, 0., 0.))
}
fn cloud(p: Vector3, u: &Uniforms) -> f32 {
    let t = u.time * 0.045;
    let q = Vector3::new(
        p.x * t.cos() + p.z * t.sin(),
        p.y,
        -p.x * t.sin() + p.z * t.cos(),
    );
    smoothstep(
        u.coverage,
        u.coverage + 0.16,
        fbm(&u.noise, q * 6. + Vector3::new(3., 1., 5.)),
    )
}
pub fn rocky(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal;
    let view = u.view_direction;
    let h = height(p, u);
    let land = smoothstep(0.44, 0.48, h);
    let e = 0.007;
    let grad = Vector3::new(
        height(p + Vector3::new(e, 0., 0.), u) - height(p - Vector3::new(e, 0., 0.), u),
        height(p + Vector3::new(0., e, 0.), u) - height(p - Vector3::new(0., e, 0.), u),
        height(p + Vector3::new(0., 0., e), u) - height(p - Vector3::new(0., 0., e), u),
    ) / (2. * e);
    let bn = (n - (grad - n * n.dot(grad)) * u.bump * land).normalized();
    let d = bn.dot(u.light).max(0.);
    let ocean = mix(
        Vector3::new(0.015, 0.08, 0.19),
        Vector3::new(0.02, 0.55, 0.61),
        smoothstep(0.3, 0.46, h),
    );
    let ground = mix(
        Vector3::new(0.15, 0.32, 0.23),
        Vector3::new(0.83, 0.49, 0.62),
        smoothstep(0.49, 0.68, h),
    );
    let mut base = mix(ocean, ground, land);
    base = mix(
        base,
        Vector3::new(0.88, 0.96, 1.),
        smoothstep(0.72, 0.91, p.y.abs() + h * 0.13),
    );
    let half = (u.light + view).normalized();
    let spec = bn.dot(half).max(0.).powf(95.) * (1. - land) * d;
    let mut color = base * (0.075 + 0.925 * d) + Vector3::new(0.8, 0.94, 1.) * spec * 1.5;
    // Misma máscara muestreada hacia la luz en una capa a altura positiva.
    if u.clouds {
        let radial = n.dot(u.light);
        let shadow_p =
            (p + (u.light - n * radial) * (0.065 / radial.max(0.18))).normalized() * p.length();
        let shadow = cloud(shadow_p, u) * smoothstep(0., 0.25, radial);
        color *= 1. - 0.48 * shadow;
    }
    let city = smoothstep(0.72, 0.82, sample(&u.noise, p * 170.) * 0.5 + 0.5)
        * smoothstep(0.47, 0.52, h)
        * (1. - smoothstep(0., 0.22, n.dot(u.light)));
    color += Vector3::new(1., 0.56, 0.22) * city * 0.8;
    if u.clouds {
        color = mix(
            color,
            console_styles::smooth(fragment, u, Vector3::new(0.84, 0.94, 1.), 0.23, 0.77),
            cloud(p, u) * 0.88,
        );
    }
    color + Vector3::new(0.08, 0.4, 0.65) * (1. - n.dot(view).max(0.)).powf(4.) * 0.38
}
pub fn gas_giant(fragment: &Fragment, u: &Uniforms) -> Vector3 {
    let p = fragment.object_position;
    let n = fragment.normal;
    let view = u.view_direction;
    let mut lon = p.x.atan2(p.z);
    let mut lat = (p.y / p.length()).asin();
    let dx = (lon - 0.35 + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let dy = lat - 0.17;
    let r = ((dx / 0.5).powi(2) + (dy / 0.32).powi(2)).sqrt();
    let angle = 3.4 * (-r * r * 1.3).exp();
    lon = 0.35 + dx * angle.cos() - dy * angle.sin();
    lat = 0.17 + dx * angle.sin() + dy * angle.cos();
    let q = Vector3::new(lon.cos() * lat.cos(), lat.sin(), lon.sin() * lat.cos());
    let flow = fbm(&u.noise, q * 8. + Vector3::new(u.time * 0.07, 0., 0.));
    let bands = (lat * 32. + flow * 6. + (lon * 5. + u.time * 0.15).sin() * 0.65).sin() * 0.5 + 0.5;
    let mut c = mix(
        Vector3::new(0.12, 0.035, 0.23),
        Vector3::new(0.82, 0.25, 0.4),
        smoothstep(0.12, 0.58, bands),
    );
    c = mix(
        c,
        Vector3::new(1., 0.77, 0.58),
        smoothstep(0.65, 0.92, bands),
    );
    let filament = (lat * 110. + flow * 14.).sin() * 0.5 + 0.5;
    c *= 0.8 + 0.2 * filament;
    c = mix(c, Vector3::new(0.96, 0.51, 0.36), (-r * r * 3.).exp() * 0.3);
    console_styles::smooth(fragment, u, c, 0.17, 0.83)
        + Vector3::new(0.3, 0.13, 0.4) * (1. - n.dot(view).max(0.)).powf(4.) * 0.3
}
