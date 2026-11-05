use raylib::prelude::*;

mod fragment;
mod framebuffer;
mod matrix;
mod obj;
mod pipeline;
mod triangle;
mod vertex;

use framebuffer::Framebuffer;

const WIDTH: i32 = 960;
const HEIGHT: i32 = 720;

pub struct Uniforms {
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
            // Material provisional para comprobar volumen e iluminación.
            let color = Vector3::new(0.18, 0.65, 1.0) * (0.12 + 0.88 * fragment.intensity);
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
    let uniforms = Uniforms {
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

    while !window.window_should_close() {
        framebuffer.clear(background);
        render(&mut framebuffer, &vertices, &uniforms);
        framebuffer.present(&mut window, &thread);
        frames += 1;
        if smoke_check && frames >= 3 {
            break;
        }
    }
}
