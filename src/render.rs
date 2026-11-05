use crate::{Uniforms, framebuffer::Framebuffer, shaders, triangle, vertex};

// Render: vértices -> triángulos -> profundidad -> fragment_shader.
// La visibilidad se guarda para reutilizarla con la misma esfera y matrices.
#[allow(clippy::chunks_exact_to_as_chunks)]
pub fn render(framebuffer: &mut Framebuffer, vertices: &[vertex::Vertex], uniforms: &Uniforms) {
    let mut signature = 14695981039346656037_u64;
    for vertex in vertices {
        for value in [
            vertex.object_position.x,
            vertex.object_position.y,
            vertex.object_position.z,
            vertex.normal.x,
            vertex.normal.y,
            vertex.normal.z,
        ] {
            signature = (signature ^ value.to_bits() as u64).wrapping_mul(1099511628211);
        }
    }
    let key = crate::framebuffer::GeometryKey {
        vertices: signature,
        model: uniforms.model_matrix,
        view: uniforms.view_matrix,
        viewport: uniforms.viewport_matrix,
    };
    if framebuffer.geometry != Some(key) {
        framebuffer.clear();
        let transformed: Vec<_> = vertices
            .iter()
            .map(|v| shaders::vertex_shader(v, uniforms))
            .collect();
        for vertices in transformed.chunks_exact(3) {
            for fragment in triangle::triangle(vertices, uniforms) {
                let index =
                    fragment.position.y as usize * framebuffer.width + fragment.position.x as usize;
                if fragment.depth < framebuffer.depths[index] {
                    framebuffer.depths[index] = fragment.depth;
                    framebuffer.fragments[index] = Some(fragment);
                }
            }
        }
        framebuffer.geometry = Some(key);
    }
    // Calcular el color del fragmento visible de cada píxel.
    // Cada hilo escribe una franja distinta; el sombreado sigue siendo en CPU.
    let chunk = framebuffer.colors.len().div_ceil(framebuffer.workers);
    std::thread::scope(|scope| {
        for (colors, fragments) in framebuffer
            .colors
            .chunks_mut(chunk)
            .zip(framebuffer.fragments.chunks(chunk))
        {
            scope.spawn(move || {
                for (color, fragment) in colors.iter_mut().zip(fragments) {
                    if let Some(mut fragment) = *fragment {
                        // La luz puede cambiar aunque la geometría esté en caché.
                        fragment.intensity = fragment.normal.dot(uniforms.light).max(0.);
                        *color = shaders::fragment_shader(&fragment, uniforms);
                    }
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use raylib::prelude::Vector3;
    // Render secuencial para comprobar la equivalencia de las optimizaciones.
    #[allow(clippy::chunks_exact_to_as_chunks)]
    fn sequential_render(
        framebuffer: &mut Framebuffer,
        vertices: &[vertex::Vertex],
        uniforms: &Uniforms,
    ) {
        framebuffer.clear();
        let transformed: Vec<_> = vertices
            .iter()
            .map(|v| shaders::vertex_shader(v, uniforms))
            .collect();
        for vertices in transformed.chunks_exact(3) {
            for fragment in triangle::triangle(vertices, uniforms) {
                let index =
                    fragment.position.y as usize * framebuffer.width + fragment.position.x as usize;
                if fragment.depth < framebuffer.depths[index] {
                    framebuffer.depths[index] = fragment.depth;
                    framebuffer.colors[index] = shaders::fragment_shader(&fragment, uniforms);
                }
            }
        }
    }
    #[test]
    fn optimized_render_matches_sequential_render_with_camera_light_and_material_changes() {
        let mut vertices = crate::obj::load_sphere();
        let mut reference = Framebuffer::new(72, 72);
        let mut optimized = Framebuffer::new(72, 72);
        optimized.workers = 4;
        let mut u = Uniforms::new(72, 72);
        for step in 0..9 {
            u.shader = step % 3;
            u.time = step as f32 * 0.2;
            u.light = Vector3::new(-0.8 + step as f32 * 0.2, 0.3, 0.5).normalized();
            u.clouds = step % 2 == 0;
            u.bump = if step % 2 == 0 { 0.12 } else { 0. };
            u.set_camera(if step < 6 { 0. } else { 0.4 });
            sequential_render(&mut reference, &vertices, &u);
            render(&mut optimized, &vertices, &u);
            reference.update_rgb();
            optimized.update_rgb();
            assert_eq!(reference.rgb, optimized.rgb);
            assert_eq!(reference.depths, optimized.depths);
        }
        // También detectar cambios hechos dentro del mismo arreglo de vértices.
        vertices[0].object_position.x += 0.05;
        sequential_render(&mut reference, &vertices, &u);
        render(&mut optimized, &vertices, &u);
        reference.update_rgb();
        optimized.update_rgb();
        assert_eq!(reference.rgb, optimized.rgb);
    }
    #[test]
    fn shared_sphere_and_temporal_material() {
        let vertices = crate::obj::load_sphere();
        assert!(!vertices.is_empty());
        assert_eq!(vertices.len() % 3, 0);
        let originals: Vec<_> = vertices.iter().map(|v| v.object_position).collect();
        let mut frame = Framebuffer::new(80, 80);
        let mut uniforms = Uniforms::new(80, 80);
        let mut images = Vec::new();
        for shader in 0..3 {
            uniforms.shader = shader;
            render(&mut frame, &vertices, &uniforms);
            assert!(frame.depths.iter().filter(|z| z.is_finite()).count() > 1000);
            assert!(
                frame
                    .colors
                    .iter()
                    .all(|c| c.x.is_finite() && c.y.is_finite() && c.z.is_finite())
            );
            frame.update_rgb();
            images.push(frame.rgb.clone());
        }
        assert_ne!(images[0], images[1]);
        assert_ne!(images[1], images[2]);
        uniforms.shader = 0;
        uniforms.time = 3.;
        render(&mut frame, &vertices, &uniforms);
        frame.update_rgb();
        assert_ne!(images[0], frame.rgb);
        assert_eq!(
            originals,
            vertices
                .iter()
                .map(|v| v.object_position)
                .collect::<Vec<_>>()
        );
    }
    #[test]
    fn z_buffer_selects_nearest_surface_in_either_submission_order() {
        let mut uniforms = Uniforms::new(32, 32);
        uniforms.shader = 1;
        uniforms.clouds = false;
        let near = [
            Vector3::new(-0.5, -0.5, 0.5),
            Vector3::new(0.5, -0.5, 0.5),
            Vector3::new(0., 0.5, 0.5),
        ]
        .map(vertex::Vertex::new);
        let far = near.map(|v| {
            vertex::Vertex::new(Vector3::new(v.object_position.x, v.object_position.y, -0.5))
        });
        let mut frame = Framebuffer::new(32, 32);
        render(&mut frame, &near, &uniforms);
        let expected = frame.colors[16 * 32 + 16];
        for vertices in [[far, near].concat(), [near, far].concat()] {
            render(&mut frame, &vertices, &uniforms);
            assert_eq!(frame.depths[16 * 32 + 16], -0.5);
            assert_eq!(frame.colors[16 * 32 + 16], expected);
        }
    }
}
