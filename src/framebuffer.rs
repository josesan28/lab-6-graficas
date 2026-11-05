use raylib::prelude::*;

// Basado en el framebuffer del laboratorio 5: la CPU prepara los píxeles
// y Raylib presenta la imagen completa mediante una textura de pantalla.
pub struct Framebuffer {
    pixels: Vec<u8>,
    texture: Texture2D,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, window: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let image = Image::gen_image_color(width, height, Color::BLACK);
        let texture = window
            .load_texture_from_image(thread, &image)
            .expect("No se pudo crear la textura del framebuffer");
        Self {
            pixels: vec![0; (width * height * 4) as usize],
            texture,
        }
    }

    pub fn clear(&mut self, color: Color) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }

    pub fn present(&mut self, window: &mut RaylibHandle, thread: &RaylibThread) {
        self.texture
            .update_texture(&self.pixels)
            .expect("No se pudo actualizar la textura del framebuffer");
        let mut drawing = window.begin_drawing(thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture(&self.texture, 0, 0, Color::WHITE);
    }
}
