use crate::{Uniforms, fragment::Fragment};
use raylib::prelude::*;

#[derive(Clone, Copy)]
pub enum Shader {
    Estrella,
    Rocoso,
    Gaseoso,
    Volcanico,
    Helado,
}

impl Shader {
    pub fn name(self) -> &'static str {
        match self {
            Self::Volcanico => "Planeta volcánico",
            Self::Helado => "Planeta helado",
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
        Shader::Volcanico => crate::volcanic::shade(fragment, uniforms),
        Shader::Helado => crate::frozen::shade(fragment, uniforms),
        Shader::Estrella => crate::star::shade(fragment, uniforms),
        Shader::Rocoso => crate::rock::shade(fragment, uniforms),
        Shader::Gaseoso => crate::gas::shade(fragment, uniforms),
    }
}
