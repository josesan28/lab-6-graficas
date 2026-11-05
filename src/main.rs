use raylib::prelude::*;

mod framebuffer;

use framebuffer::Framebuffer;

const WIDTH: i32 = 960;
const HEIGHT: i32 = 720;

fn main() {
    let (mut window, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Laboratorio 6 - Shaders")
        .build();
    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, &mut window, &thread);
    let background = Color::new(9, 12, 35, 255);

    let smoke_check = std::env::args().any(|arg| arg == "--smoke-check");
    let mut frames = 0;

    while !window.window_should_close() {
        framebuffer.clear(background);
        framebuffer.present(&mut window, &thread);
        frames += 1;
        if smoke_check && frames >= 3 {
            break;
        }
    }
}
