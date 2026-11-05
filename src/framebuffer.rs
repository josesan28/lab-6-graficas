use raylib::prelude::*;

pub struct Framebuffer {
    width: usize,
    depths: Vec<f32>,
    pixels: Vec<u8>,
    texture: Texture2D,
    hud: crate::hud::Hud,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32, window: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        let image = Image::gen_image_color(width, height, Color::BLACK);
        let texture = window
            .load_texture_from_image(thread, &image)
            .expect("No se pudo crear la textura del framebuffer");
        let hud = crate::hud::Hud::new(window, thread);
        Self {
            width: width as usize,
            depths: vec![f32::INFINITY; (width * height) as usize],
            pixels: vec![0; (width * height * 4) as usize],
            texture,
            hud,
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.depths.fill(f32::INFINITY);
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
    }

    pub fn point(&mut self, x: usize, y: usize, depth: f32, color: Vector3) {
        let index = y * self.width + x;
        if depth < self.depths[index] {
            self.depths[index] = depth;
            self.pixels[index * 4..index * 4 + 4].copy_from_slice(&[
                (color.x.clamp(0.0, 1.0) * 255.0) as u8,
                (color.y.clamp(0.0, 1.0) * 255.0) as u8,
                (color.z.clamp(0.0, 1.0) * 255.0) as u8,
                255,
            ]);
        }
    }

    pub fn save(&self, path: &str) {
        let height = self.depths.len() / self.width;
        let mut image = Image::gen_image_color(self.width as i32, height as i32, Color::BLACK);
        for (index, pixel) in self.pixels.chunks_exact(4).enumerate() {
            image.draw_pixel(
                (index % self.width) as i32,
                (index / self.width) as i32,
                Color::new(pixel[0], pixel[1], pixel[2], pixel[3]),
            );
        }
        image.export_image(path);
        assert!(
            std::path::Path::new(path).is_file(),
            "No se pudo guardar la captura"
        );
    }

    pub fn present(
        &mut self,
        window: &mut RaylibHandle,
        thread: &RaylibThread,
        uniforms: &crate::Uniforms,
        paused: bool,
    ) {
        self.texture
            .update_texture(&self.pixels)
            .expect("No se pudo actualizar la textura del framebuffer");
        let mut drawing = window.begin_drawing(thread);
        drawing.clear_background(Color::BLACK);
        drawing.draw_texture(&self.texture, 0, 0, Color::WHITE);
        crate::hud::draw(&mut drawing, &self.hud, self.width as i32, uniforms, paused);
    }
}
