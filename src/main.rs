mod math;
mod renderer;
mod shaders;
use math::*;
use raylib::prelude::*;
use renderer::{Frame, Mesh};
use shaders::*;
fn main() {
    let mesh = Mesh::load();
    let mut frame = Frame::new(600, 600);
    let mut u = Params::default();
    let (mut rl, thread) = raylib::init()
        .size(1000, 760)
        .title("Lab 6 | IAN - JIWOO - RUKA | CPU shaders")
        .build();
    rl.set_target_fps(60);
    let mut image = Image::gen_image_color(frame.w as i32, frame.h as i32, Color::BLACK);
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
        u.light = V(
            -0.65 * light_angle.cos() + 0.65 * light_angle.sin(),
            0.4,
            0.65 * light_angle.cos() + 0.65 * light_angle.sin(),
        )
        .unit();
        frame.render(&mesh, kind, &u, yaw);
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
