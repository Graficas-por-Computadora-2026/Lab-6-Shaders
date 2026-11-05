mod fragment;
mod framebuffer;
mod obj;
mod render;
mod shaders;
mod triangle;
mod vertex;
use framebuffer::Framebuffer;
use noise::OpenSimplex;
use raylib::prelude::*;
use shaders::NAMES;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub view_direction: Vector3,
    pub light: Vector3,
    pub time: f32,
    pub noise: OpenSimplex,
    pub shader: usize,
    pub clouds: bool,
    pub coverage: f32,
    pub bump: f32,
    pub width: usize,
    pub height: usize,
}
impl Uniforms {
    pub fn new(width: usize, height: usize) -> Self {
        let mut result = Self {
            model_matrix: Matrix::identity(),
            view_matrix: Matrix::identity(),
            viewport_matrix: Matrix::identity(),
            view_direction: Vector3::new(0., 0., 1.),
            light: Vector3::new(-0.65, 0.4, 0.65).normalized(),
            time: 0.,
            noise: OpenSimplex::new(42),
            shader: 0,
            clouds: true,
            coverage: 0.48,
            bump: 0.12,
            width,
            height,
        };
        result.set_camera(0.);
        result
    }
    pub fn set_camera(&mut self, yaw: f32) {
        self.view_direction = Vector3::new(yaw.sin(), 0., yaw.cos());
        let mut view = Matrix::identity();
        view.m0 = yaw.cos();
        view.m8 = -yaw.sin();
        // Una profundidad menor corresponde a una superficie más cercana.
        view.m2 = -yaw.sin();
        view.m10 = -yaw.cos();
        self.view_matrix = view;
        let mut viewport = Matrix::identity();
        viewport.m0 = self.height as f32 * 0.48;
        viewport.m5 = -viewport.m0;
        viewport.m12 = self.width as f32 * 0.5;
        viewport.m13 = self.height as f32 * 0.5;
        self.viewport_matrix = viewport;
    }
}
fn main() {
    let vertices = obj::load_sphere();
    let mut frame = Framebuffer::new(600, 600);
    let mut u = Uniforms::new(600, 600);
    let (mut rl, thread) = raylib::init()
        .size(1000, 760)
        .title("Lab 6 | IAN - JIWOO - RUKA | CPU shaders")
        .build();
    rl.set_target_fps(60);
    let mut image = Image::gen_image_color(frame.width as i32, frame.height as i32, Color::BLACK);
    // El framebuffer tiene tres bytes RGB por píxel; raylib crea RGBA por defecto.
    image.set_format(PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8);
    let mut texture = rl.load_texture_from_image(&thread, &image).unwrap();
    let mut kind = 0;
    let mut paused = false;
    let mut yaw = 0.;
    let mut light_angle = 0.;
    while !rl.window_should_close() {
        let dt = rl.get_frame_time().min(0.1);
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            kind = 0;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            kind = 1;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_THREE) {
            kind = 2;
        }
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
            paused = !paused;
        }
        if kind == 1 && rl.is_key_pressed(KeyboardKey::KEY_C) {
            u.clouds = !u.clouds;
        }
        if rl.is_key_down(KeyboardKey::KEY_LEFT) {
            yaw -= dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
            yaw += dt;
        }
        if kind != 0 && rl.is_key_down(KeyboardKey::KEY_L) {
            light_angle += dt;
        }
        if kind == 1 && u.clouds && rl.is_key_down(KeyboardKey::KEY_UP) {
            u.coverage = (u.coverage - dt * 0.1).max(0.2);
        }
        if kind == 1 && u.clouds && rl.is_key_down(KeyboardKey::KEY_DOWN) {
            u.coverage = (u.coverage + dt * 0.1).min(0.8);
        }
        if kind == 1 && rl.is_key_pressed(KeyboardKey::KEY_B) {
            u.bump = if u.bump > 0. { 0. } else { 0.12 };
        }
        if !paused {
            u.time += dt;
        }
        u.light = Vector3::new(
            -0.65 * light_angle.cos() + 0.65 * light_angle.sin(),
            0.4,
            0.65 * light_angle.cos() + 0.65 * light_angle.sin(),
        )
        .normalized();
        u.shader = kind;
        u.set_camera(yaw);
        render::render(&mut frame, &vertices, &u);
        frame.update_rgb();
        texture.update_texture(&frame.rgb).unwrap();
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::new(7, 10, 22, 255));
        d.draw_text(
            "CELESTIAL STAGE",
            34,
            28,
            30,
            Color::new(245, 205, 176, 255),
        );
        d.draw_text(NAMES[kind], 34, 70, 22, Color::WHITE);
        d.draw_fps(850, 30);
        d.draw_texture(&texture, 200, 110, Color::WHITE);
        d.draw_text(
            "1 IAN   2 JIWOO   3 RUKA   |   SPACE pausa   |   <- -> camara",
            34,
            715,
            18,
            Color::WHITE,
        );
        let effects = match kind {
            1 if u.clouds => "L luz | C desactivar nubes | arriba/abajo cobertura | B relieve",
            1 => "L luz | C activar nubes | B relieve",
            2 => "L mover luz",
            _ => "",
        };
        d.draw_text(effects, 34, 740, 16, Color::LIGHTGRAY);
    }
}
