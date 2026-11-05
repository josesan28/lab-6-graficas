use raylib::prelude::*;

mod clouds;
mod controls;
mod fragment;
mod framebuffer;
mod gas;
mod matrix;
mod night;
mod obj;
mod patterns;
mod pipeline;
mod rock;
mod shaders;
mod star;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;

const WIDTH: i32 = 960;
const HEIGHT: i32 = 720;

pub struct Uniforms {
    pub noise: noise::OpenSimplex,
    pub time: f32,
    pub camera_position: Vector3,
    pub shader: shaders::Shader,
    pub show_normals: bool,
    pub specular_enabled: bool,
    pub clouds_enabled: bool,
    pub cloud_coverage: f32,
    pub night_lights_enabled: bool,
    pub vortex_enabled: bool,
    pub bump_enabled: bool,
    pub light_angle: f32,
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub resolution: Vector2,
    pub light_position: Vector3,
    pub smooth_normals: bool,
}

fn render(framebuffer: &mut Framebuffer, vertices: &[vertex::Vertex], uniforms: &Uniforms) {
    let transformed: Vec<_> = vertices
        .iter()
        .map(|vertex| pipeline::vertex_shader(vertex, uniforms))
        .collect();
    for face in transformed.chunks_exact(3) {
        for fragment in triangle::triangle(face, uniforms) {
            let color = shaders::fragment_shader(&fragment, uniforms);
            framebuffer.point(
                fragment.position.x as usize,
                fragment.position.y as usize,
                fragment.depth,
                color,
            );
        }
    }
}

fn main() {
    let model_path = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models/sphere.obj");
    let vertices = obj::load(model_path).expect("No se pudo cargar la esfera original");
    println!("Esfera: {} triángulos", vertices.len() / 3);
    let mut uniforms = Uniforms {
        noise: noise::OpenSimplex::new(42),
        time: 0.0,
        camera_position: Vector3::new(0.0, 0.0, 3.2),
        shader: shaders::Shader::Rocoso,
        show_normals: false,
        specular_enabled: true,
        clouds_enabled: true,
        cloud_coverage: 0.45,
        night_lights_enabled: true,
        vortex_enabled: true,
        bump_enabled: true,
        light_angle: (-3.0_f32).atan2(5.0),
        model_matrix: Matrix::identity(),
        view_matrix: matrix::create_view_matrix(
            Vector3::new(0.0, 0.0, 3.2),
            Vector3::zero(),
            Vector3::new(0.0, 1.0, 0.0),
        ),
        projection_matrix: matrix::create_projection_matrix(
            std::f32::consts::PI / 3.0,
            WIDTH as f32 / HEIGHT as f32,
            0.1,
            100.0,
        ),
        viewport_matrix: matrix::create_viewport_matrix(0.0, 0.0, WIDTH as f32, HEIGHT as f32),
        resolution: Vector2::new(WIDTH as f32, HEIGHT as f32),
        light_position: Vector3::new(-3.0, 4.0, 5.0),
        smooth_normals: true,
    };
    let (mut window, thread) = raylib::init()
        .size(WIDTH, HEIGHT)
        .title("Laboratorio 6 - Shaders")
        .build();
    window.set_target_fps(60);

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, &mut window, &thread);
    let background = Color::new(9, 12, 35, 255);

    let smoke_check = std::env::args().any(|arg| arg == "--smoke-check");
    let mut frames = 0;
    let mut controls = controls::Controls::new();
    let arguments: Vec<String> = std::env::args().collect();
    let argument = |flag: &str| {
        arguments.iter().position(|arg| arg == flag).map(|index| {
            arguments
                .get(index + 1)
                .unwrap_or_else(|| panic!("Falta el valor de {flag}"))
        })
    };
    if let Some(shader) = argument("--shader") {
        uniforms.shader = match shader.as_str() {
            "1" => shaders::Shader::Estrella,
            "2" => shaders::Shader::Rocoso,
            "3" => shaders::Shader::Gaseoso,
            _ => panic!("El shader debe ser 1, 2 o 3"),
        };
    }
    let capture = argument("--capture");
    uniforms.specular_enabled = !arguments.iter().any(|arg| arg == "--no-specular");
    uniforms.clouds_enabled = !arguments.iter().any(|arg| arg == "--no-clouds");
    uniforms.night_lights_enabled = !arguments.iter().any(|arg| arg == "--no-night-lights");
    uniforms.vortex_enabled = !arguments.iter().any(|arg| arg == "--no-vortex");
    uniforms.bump_enabled = !arguments.iter().any(|arg| arg == "--no-bump");
    if let Some(value) = argument("--light-angle") {
        let degrees: f32 = value.parse().expect("El ángulo debe ser un número");
        assert!(degrees.is_finite(), "El ángulo debe ser finito");
        uniforms.light_angle = degrees.rem_euclid(360.0).to_radians();
    }
    if let Some(value) = argument("--cloud-coverage") {
        let coverage: f32 = value.parse().expect("La cobertura debe ser un número");
        assert!(
            (0.0..=1.0).contains(&coverage),
            "La cobertura debe estar entre 0 y 1"
        );
        uniforms.cloud_coverage = coverage;
    }
    let fixed_time = argument("--time").map(|value| {
        let time: f32 = value.parse().expect("El tiempo debe ser un número");
        assert!(
            time.is_finite() && time >= 0.0,
            "El tiempo debe ser finito y no negativo"
        );
        time
    });

    while !window.window_should_close() {
        controls.update(&window, &mut uniforms);
        if let Some(time) = fixed_time {
            uniforms.time = time;
        }
        window.set_window_title(
            &thread,
            &format!(
                "{} | {} t={:.1}s | N:{} E:{} C:{} {:.0}% V:{} T:{} B:{} | Luz:{:.0}°",
                uniforms.shader.name(),
                if controls.paused {
                    "Pausado"
                } else {
                    "En marcha"
                },
                uniforms.time,
                if uniforms.show_normals { "sí" } else { "no" },
                if uniforms.specular_enabled {
                    "sí"
                } else {
                    "no"
                },
                if uniforms.clouds_enabled { "sí" } else { "no" },
                uniforms.cloud_coverage * 100.0,
                if uniforms.night_lights_enabled {
                    "sí"
                } else {
                    "no"
                },
                if uniforms.vortex_enabled { "sí" } else { "no" },
                if uniforms.bump_enabled { "sí" } else { "no" },
                uniforms.light_angle.to_degrees(),
            ),
        );
        framebuffer.clear(background);
        render(&mut framebuffer, &vertices, &uniforms);
        if let Some(path) = capture {
            framebuffer.save(path);
            break;
        }
        framebuffer.present(&mut window, &thread);
        frames += 1;
        if smoke_check && frames >= 3 {
            break;
        }
    }
}
