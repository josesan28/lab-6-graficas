#![allow(dead_code)]

use raylib::prelude::*;

/// Crea una matriz 4x4 a partir de 16 valores indicados por filas.
pub fn new_matrix4(
    // Fila 0
    r0c0: f32,
    r0c1: f32,
    r0c2: f32,
    r0c3: f32,
    // Fila 1
    r1c0: f32,
    r1c1: f32,
    r1c2: f32,
    r1c3: f32,
    // Fila 2
    r2c0: f32,
    r2c1: f32,
    r2c2: f32,
    r2c3: f32,
    // Fila 3
    r3c0: f32,
    r3c1: f32,
    r3c2: f32,
    r3c3: f32,
) -> Matrix {
    // Distribuye los valores indicados por filas en los campos de Matrix, organizados por columnas.
    Matrix {
        // Columna 0.
        m0: r0c0,
        m1: r1c0,
        m2: r2c0,
        m3: r3c0,
        // Columna 1.
        m4: r0c1,
        m5: r1c1,
        m6: r2c1,
        m7: r3c1,
        // Columna 2.
        m8: r0c2,
        m9: r1c2,
        m10: r2c2,
        m11: r3c2,
        // Columna 3.
        m12: r0c3,
        m13: r1c3,
        m14: r2c3,
        m15: r3c3,
    }
}

/// Crea una matriz de transformación 4x4 a partir de una matriz 3x3 indicada por filas.
pub fn new_matrix3(
    // Fila 0
    r0c0: f32,
    r0c1: f32,
    r0c2: f32,
    // Fila 1
    r1c0: f32,
    r1c1: f32,
    r1c2: f32,
    // Fila 2
    r2c0: f32,
    r2c1: f32,
    r2c2: f32,
) -> Matrix {
    new_matrix4(
        r0c0, r0c1, r0c2, 0.0, r1c0, r1c1, r1c2, 0.0, r2c0, r2c1, r2c2, 0.0, 0.0, 0.0, 0.0, 1.0,
    )
}

/// Crea la matriz de modelo combinando traslación, escala y rotación.
pub fn create_model_matrix(translation: Vector3, scale: f32, rotation: Vector3) -> Matrix {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    // Rotación alrededor del eje X.
    let rotation_matrix_x = new_matrix4(
        1.0, 0.0, 0.0, 0.0, 0.0, cos_x, -sin_x, 0.0, 0.0, sin_x, cos_x, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    // Rotación alrededor del eje Y.
    let rotation_matrix_y = new_matrix4(
        cos_y, 0.0, sin_y, 0.0, 0.0, 1.0, 0.0, 0.0, -sin_y, 0.0, cos_y, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    // Rotación alrededor del eje Z.
    let rotation_matrix_z = new_matrix4(
        cos_z, -sin_z, 0.0, 0.0, sin_z, cos_z, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    // Matriz de escala.
    let scale_matrix = new_matrix4(
        scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, 0.0, 1.0,
    );

    // Matriz de traslación.
    let translation_matrix = new_matrix4(
        1.0,
        0.0,
        0.0,
        translation.x,
        0.0,
        1.0,
        0.0,
        translation.y,
        0.0,
        0.0,
        1.0,
        translation.z,
        0.0,
        0.0,
        0.0,
        1.0,
    );

    translation_matrix * rotation_matrix * scale_matrix
}

/// Crea la matriz de vista usando la posición, el objetivo y el vector vertical de la cámara.
/// Implementa una matriz lookAt para transformar el mundo al espacio de la cámara.
pub fn create_view_matrix(eye: Vector3, target: Vector3, up: Vector3) -> Matrix {
    // Calcula el vector que apunta desde la cámara hacia su objetivo.
    let mut forward = Vector3::new(target.x - eye.x, target.y - eye.y, target.z - eye.z);
    // Normaliza el vector de dirección.
    let forward_length =
        (forward.x * forward.x + forward.y * forward.y + forward.z * forward.z).sqrt();
    forward.x /= forward_length;
    forward.y /= forward_length;
    forward.z /= forward_length;

    // Calcula el vector lateral mediante el producto cruz de la dirección y el vector vertical.
    let mut right = Vector3::new(
        forward.y * up.z - forward.z * up.y,
        forward.z * up.x - forward.x * up.z,
        forward.x * up.y - forward.y * up.x,
    );
    // Normaliza el vector lateral.
    let right_length = (right.x * right.x + right.y * right.y + right.z * right.z).sqrt();
    right.x /= right_length;
    right.y /= right_length;
    right.z /= right_length;

    // Calcula el vector vertical perpendicular a los vectores lateral y de dirección.
    let actual_up = Vector3::new(
        right.y * forward.z - right.z * forward.y,
        right.z * forward.x - right.x * forward.z,
        right.x * forward.y - right.y * forward.x,
    );

    // Construye la matriz de vista como inversa de la transformación de la cámara.
    // Utiliza la fórmula de la matriz lookAt.
    new_matrix4(
        right.x,
        right.y,
        right.z,
        -(right.x * eye.x + right.y * eye.y + right.z * eye.z),
        actual_up.x,
        actual_up.y,
        actual_up.z,
        -(actual_up.x * eye.x + actual_up.y * eye.y + actual_up.z * eye.z),
        -forward.x,
        -forward.y,
        -forward.z,
        forward.x * eye.x + forward.y * eye.y + forward.z * eye.z,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

/// Crea una matriz de proyección en perspectiva.
/// fov_y: campo de visión vertical en radianes.
/// aspect: relación entre el ancho y el alto.
/// near: distancia al plano de recorte cercano.
/// far: distancia al plano de recorte lejano.
pub fn create_projection_matrix(fov_y: f32, aspect: f32, near: f32, far: f32) -> Matrix {
    let tan_half_fov = (fov_y / 2.0).tan();

    new_matrix4(
        1.0 / (aspect * tan_half_fov),
        0.0,
        0.0,
        0.0,
        0.0,
        1.0 / tan_half_fov,
        0.0,
        0.0,
        0.0,
        0.0,
        -(far + near) / (far - near),
        -(2.0 * far * near) / (far - near),
        0.0,
        0.0,
        -1.0,
        0.0,
    )
}

/// Crea la matriz que convierte coordenadas normalizadas en coordenadas de pantalla.
/// x, y: posición del área de dibujo, normalmente (0, 0).
/// width, height: dimensiones del área de dibujo en píxeles.
pub fn create_viewport_matrix(x: f32, y: f32, width: f32, height: f32) -> Matrix {
    let half_width = width / 2.0;
    let half_height = height / 2.0;

    new_matrix4(
        half_width,
        0.0,
        0.0,
        x + half_width,
        0.0,
        -half_height,
        0.0,
        y + half_height,
        0.0,
        0.0,
        1.0,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
