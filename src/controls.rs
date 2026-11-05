use crate::{Uniforms, matrix, shaders::Shader};
use raylib::prelude::*;

pub struct Controls {
    yaw: f32,
    pitch: f32,
    distance: f32,
    pub paused: bool,
}

impl Controls {
    pub fn set_distance(&mut self, distance: f32) {
        self.distance = distance.clamp(2.2, 6.0);
    }

    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.2,
            paused: false,
        }
    }

    pub fn update(&mut self, window: &RaylibHandle, uniforms: &mut Uniforms) {
        let delta = window.get_frame_time();
        if window.is_key_pressed(KeyboardKey::KEY_ONE) {
            uniforms.shader = Shader::Estrella;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TWO) {
            uniforms.shader = Shader::Rocoso;
        }
        if window.is_key_pressed(KeyboardKey::KEY_THREE) {
            uniforms.shader = Shader::Gaseoso;
        }
        if window.is_key_pressed(KeyboardKey::KEY_N) {
            uniforms.show_normals = !uniforms.show_normals;
        }
        if window.is_key_pressed(KeyboardKey::KEY_E) {
            uniforms.specular_enabled = !uniforms.specular_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_C) {
            uniforms.clouds_enabled = !uniforms.clouds_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_V) {
            uniforms.night_lights_enabled = !uniforms.night_lights_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_T) {
            uniforms.vortex_enabled = !uniforms.vortex_enabled;
        }
        if window.is_key_pressed(KeyboardKey::KEY_B) {
            uniforms.bump_enabled = !uniforms.bump_enabled;
        }
        if window.is_key_down(KeyboardKey::KEY_J) {
            uniforms.light_angle -= delta;
        }
        if window.is_key_down(KeyboardKey::KEY_L) {
            uniforms.light_angle += delta;
        }
        if window.is_key_down(KeyboardKey::KEY_Z) {
            uniforms.cloud_coverage -= delta * 0.25;
        }
        if window.is_key_down(KeyboardKey::KEY_X) {
            uniforms.cloud_coverage += delta * 0.25;
        }
        uniforms.cloud_coverage = uniforms.cloud_coverage.clamp(0.0, 1.0);
        if window.is_key_pressed(KeyboardKey::KEY_SPACE) {
            self.paused = !self.paused;
        }

        // La velocidad depende de segundos transcurridos, no de la cantidad de cuadros.
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.yaw -= delta;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.yaw += delta;
        }
        if window.is_key_down(KeyboardKey::KEY_W) {
            self.pitch += delta;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.pitch -= delta;
        }
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.distance -= delta * 1.5;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.distance += delta * 1.5;
        }

        // Mantiene la esfera delante del plano cercano y evita invertir la cámara.
        self.pitch = self.pitch.clamp(-1.3, 1.3);
        self.distance = self.distance.clamp(2.2, 6.0);
        if !self.paused {
            uniforms.time += delta;
        }

        if window.is_key_pressed(KeyboardKey::KEY_HOME) {
            *self = Self::new();
            uniforms.time = 0.0;
            uniforms.show_normals = false;
            uniforms.light_angle = (-3.0_f32).atan2(5.0);
        }

        // La luz gira alrededor del planeta sin cambiar su altura.
        uniforms.light_angle = uniforms.light_angle.rem_euclid(std::f32::consts::TAU);
        let radius = 34.0_f32.sqrt();
        uniforms.light_position = Vector3::new(
            radius * uniforms.light_angle.sin(),
            4.0,
            radius * uniforms.light_angle.cos(),
        );

        uniforms.camera_position = Vector3::new(
            self.distance * self.pitch.cos() * self.yaw.sin(),
            self.distance * self.pitch.sin(),
            self.distance * self.pitch.cos() * self.yaw.cos(),
        );
        uniforms.view_matrix = matrix::create_view_matrix(
            uniforms.camera_position,
            Vector3::zero(),
            Vector3::new(0.0, 1.0, 0.0),
        );
    }
}
