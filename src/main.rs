use raylib::prelude::*;

mod clouds;
mod controls;
mod fragment;
mod framebuffer;
mod frozen;
mod gas;
mod hud;
mod matrix;
mod night;
mod obj;
mod patterns;
mod pipeline;
mod rock;
mod shaders;
mod star;
mod surface;
mod triangle;
mod vertex;
mod volcanic;

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
    pub parallax_enabled: bool,
    pub cloud_shadows_enabled: bool,
    pub aurora_enabled: bool,
    pub light_angle: f32,
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub resolution: Vector2,
    pub light_position: Vector3,
    pub smooth_normals: bool,
}

fn render(
    framebuffer: &mut Framebuffer,
    vertices: &[vertex::Vertex],
    uniforms: &Uniforms,
    fragments: &mut Vec<Vec<fragment::Fragment>>,
) {
    let transformed: Vec<_> = vertices
        .iter()
        .map(|vertex| pipeline::vertex_shader(vertex, uniforms))
        .collect();
    let workers = std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(8);
    fragments.resize_with(workers, Vec::new);
    // Repartir caras intercaladas balancea ambos hemisferios. Cada hilo conserva
    // su memoria entre cuadros; Raylib y el z-buffer quedan en el hilo principal.
    std::thread::scope(|scope| {
        for (worker, buffer) in fragments.iter_mut().enumerate() {
            let transformed = &transformed;
            scope.spawn(move || {
                buffer.clear();
                for face in transformed.chunks_exact(3).skip(worker).step_by(workers) {
                    triangle::triangle(face, uniforms, |mut fragment| {
                        fragment.color = shaders::fragment_shader(&fragment, uniforms);
                        buffer.push(fragment);
                    });
                }
            });
        }
    });
    for buffer in fragments {
        for fragment in buffer {
            framebuffer.point(
                fragment.position.x as usize,
                fragment.position.y as usize,
                fragment.depth,
                fragment.color,
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
        parallax_enabled: true,
        cloud_shadows_enabled: true,
        aurora_enabled: true,
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
        .size(WIDTH + hud::WIDTH, HEIGHT)
        .title("Laboratorio 6 - Shaders")
        .build();
    window.set_target_fps(60);

    rock::prepare(&uniforms);
    clouds::prepare(&uniforms);
    gas::prepare(&uniforms);
    star::prepare(&uniforms);
    volcanic::prepare(&uniforms);
    frozen::prepare(&uniforms);
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, &mut window, &thread);
    let mut fragments = Vec::new();
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
            "4" => shaders::Shader::Volcanico,
            "5" => shaders::Shader::Helado,
            _ => panic!("El shader debe estar entre 1 y 5"),
        };
    }
    let capture = argument("--capture");
    let capture_window = argument("--capture-window");
    uniforms.specular_enabled = !arguments.iter().any(|arg| arg == "--no-specular");
    uniforms.clouds_enabled = !arguments.iter().any(|arg| arg == "--no-clouds");
    uniforms.night_lights_enabled = !arguments.iter().any(|arg| arg == "--no-night-lights");
    uniforms.vortex_enabled = !arguments.iter().any(|arg| arg == "--no-vortex");
    uniforms.bump_enabled = !arguments.iter().any(|arg| arg == "--no-bump");
    uniforms.parallax_enabled = !arguments.iter().any(|arg| arg == "--no-parallax");
    uniforms.cloud_shadows_enabled = !arguments.iter().any(|arg| arg == "--no-cloud-shadows");
    uniforms.aurora_enabled = !arguments.iter().any(|arg| arg == "--no-aurora");
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

    if let Some(value) = argument("--distance") {
        let distance: f32 = value.parse().expect("La distancia debe ser un número");
        assert!(distance.is_finite(), "La distancia debe ser finita");
        controls.set_distance(distance);
    }
    let benchmark_window = arguments.iter().any(|arg| arg == "--benchmark-window");
    if let Some(directory) = argument("--capture-demo") {
        std::fs::create_dir_all(directory).expect("No se pudo crear la carpeta de evidencia");
        for frame in 0..16 {
            let phase = frame as f32 / 15.0;
            match uniforms.shader {
                shaders::Shader::Volcanico => controls.set_yaw(-0.30 + phase * 0.60),
                _ => uniforms.light_angle = -1.2 + phase * 1.6,
            }
            controls.update(&window, &mut uniforms);
            // Cámara con tiempo congelado para parallax; luz móvil para sombras.
            uniforms.time = fixed_time.unwrap_or(2.0);
            framebuffer.clear(background);
            render(&mut framebuffer, &vertices, &uniforms, &mut fragments);
            let path = std::path::Path::new(directory).join(format!("frame-{frame:02}.png"));
            framebuffer.save(path.to_str().expect("Ruta de captura inválida"));
        }
        return;
    }
    if benchmark_window || arguments.iter().any(|arg| arg == "--benchmark") {
        window.set_target_fps(0);
        controls.update(&window, &mut uniforms);
        uniforms.time = fixed_time.unwrap_or(2.0);
        let mut samples = Vec::new();
        let mut complete_samples = Vec::new();
        for index in 0..10 {
            let start = std::time::Instant::now();
            framebuffer.clear(background);
            render(&mut framebuffer, &vertices, &uniforms, &mut fragments);
            let cpu_ms = start.elapsed().as_secs_f64() * 1000.0;
            if benchmark_window {
                framebuffer.present(&mut window, &thread, &uniforms, controls.paused);
            }
            if index > 0 {
                samples.push(cpu_ms);
                complete_samples.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        samples.sort_by(f64::total_cmp);
        println!("Mediana CPU (9 cuadros): {:.2} ms", samples[4]);
        if benchmark_window {
            complete_samples.sort_by(f64::total_cmp);
            println!(
                "Mediana con presentación y HUD: {:.2} ms",
                complete_samples[4]
            );
        }
        if let Some(path) = capture {
            framebuffer.save(path);
        }
        return;
    }

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
        render(&mut framebuffer, &vertices, &uniforms, &mut fragments);
        if let Some(path) = capture {
            framebuffer.save(path);
            break;
        }
        framebuffer.present(&mut window, &thread, &uniforms, controls.paused);
        if let Some(path) = capture_window.filter(|_| frames >= 2) {
            window.take_screenshot(&thread, path);
            break;
        }
        frames += 1;
        if smoke_check && frames >= 3 {
            break;
        }
    }
}
