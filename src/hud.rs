use crate::{Uniforms, shaders::Shader};
use raylib::prelude::*;

pub const WIDTH: i32 = 360;

fn state(enabled: bool) -> &'static str {
    if enabled { "Sí" } else { "No" }
}

pub struct Hud {
    font: Option<Font>,
}
impl Hud {
    pub fn new(window: &mut RaylibHandle, thread: &RaylibThread) -> Self {
        // Fuente del sistema: no se redistribuyen archivos tipográficos propietarios.
        let windows = std::env::var("WINDIR").unwrap_or_else(|_| "C:/Windows".into());
        let candidates = [
            format!("{windows}/Fonts/segoeui.ttf"),
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf".into(),
            "/System/Library/Fonts/Supplemental/Arial.ttf".into(),
        ];
        let font = candidates.iter().find_map(|path| {
            let bytes = std::fs::read(path).ok()?;
            // Codepoints explícitos: incluye las tildes sin contar bytes UTF-8 como glifos.
            let mut glyphs: Vec<i32> = (32..=255).chain([0x2191, 0x2193]).collect();
            let raw = unsafe {
                raylib::ffi::LoadFontFromMemory(
                    c".ttf".as_ptr(),
                    bytes.as_ptr(),
                    bytes.len() as i32,
                    32,
                    glyphs.as_mut_ptr(),
                    glyphs.len() as i32,
                )
            };
            if raw.texture.id == 0 {
                return None;
            }
            let font = unsafe { Font::from_raw(raw) };
            font.texture()
                .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
            Some(font)
        });
        let _ = window;
        Self { font }
    }
}

pub fn draw(drawing: &mut RaylibDrawHandle, hud: &Hud, x: i32, uniforms: &Uniforms, paused: bool) {
    let background = Color::new(10, 14, 24, 255);
    let muted = Color::new(151, 163, 181, 255);
    let text = Color::new(184, 195, 210, 255);
    drawing.draw_rectangle(x, 0, WIDTH, 720, background);
    drawing.draw_line(x, 24, x, 696, Color::new(30, 38, 53, 255));
    let write =
        |d: &mut RaylibDrawHandle, label: &str, px: i32, y: i32, size: f32, color: Color| {
            if let Some(font) = &hud.font {
                d.draw_text_ex(
                    font,
                    label,
                    Vector2::new(px as f32, y as f32),
                    size,
                    0.0,
                    color,
                );
            } else {
                d.draw_text(label, px, y, size as i32, color);
            }
        };
    write(drawing, uniforms.shader.name(), x + 22, 28, 25.0, text);
    let status = format!(
        "{} · {:.1} s",
        if paused { "Pausado" } else { "En marcha" },
        uniforms.time
    );
    write(drawing, &status, x + 22, 59, 18.0, muted);
    let mut y = 95;
    let mut row = |key: &str, label: &str, heading: bool| {
        if heading {
            y += 14;
            write(drawing, label, x + 22, y, 18.0, muted);
            y += 29;
        } else {
            write(drawing, key, x + 22, y, 18.0, muted);
            write(drawing, label, x + 152, y, 19.0, text);
            y += 26;
        }
    };
    row("", "Cuerpos celestes", true);
    row("1", "Estrella", false);
    row("2", "Planeta rocoso", false);
    row("3", "Gigante gaseoso", false);
    row("", "Cámara y luz", true);
    row("W / S", "Orbitar vertical", false);
    row("A / D", "Orbitar horizontal", false);
    row("Arriba / Abajo", "Acercar / alejar", false);
    row("J / L", "Girar la luz", false);
    row("R", "Reiniciar", false);
    row("", "Visualización", true);
    row(
        "N",
        &format!("Normales: {}", state(uniforms.show_normals)),
        false,
    );
    row("Espacio", if paused { "Reanudar" } else { "Pausar" }, false);
    match uniforms.shader {
        Shader::Rocoso => {
            row("", "Superficie y atmósfera", true);
            row(
                "E",
                &format!("Reflejos: {}", state(uniforms.specular_enabled)),
                false,
            );
            row(
                "C",
                &format!("Nubes: {}", state(uniforms.clouds_enabled)),
                false,
            );
            row(
                "Z / X",
                &format!("Cobertura: {:.0}%", uniforms.cloud_coverage * 100.0),
                false,
            );
            row(
                "V",
                &format!("Luces: {}", state(uniforms.night_lights_enabled)),
                false,
            );
            row(
                "B",
                &format!("Relieve: {}", state(uniforms.bump_enabled)),
                false,
            );
        }
        Shader::Gaseoso => {
            row("", "Atmósfera", true);
            row(
                "T",
                &format!("Vórtice: {}", state(uniforms.vortex_enabled)),
                false,
            );
        }
        Shader::Estrella => {
            row("", "Superficie estelar animada", true);
        }
    }
    write(drawing, "Esc", x + 22, 686, 18.0, muted);
    write(drawing, "Salir", x + 152, 686, 18.0, muted);
}
