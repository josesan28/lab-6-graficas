use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

#[derive(Clone, Copy)]
pub enum Shader {
    Estrella,
    Rocoso,
    Gaseoso,
}

impl Shader {
    pub fn name(self) -> &'static str {
        match self {
            Self::Estrella => "Estrella",
            Self::Rocoso => "Planeta rocoso",
            Self::Gaseoso => "Gigante gaseoso",
        }
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    if uniforms.show_normals {
        return fragment.normal * 0.5 + Vector3::new(0.5, 0.5, 0.5);
    }

    match uniforms.shader {
        Shader::Estrella => crate::star::shade(fragment, uniforms),
        Shader::Rocoso => crate::rock::shade(fragment, uniforms),
        Shader::Gaseoso => Vector3::new(0.7, 0.35, 0.9) * (0.12 + 0.88 * fragment.intensity),
    }
}
