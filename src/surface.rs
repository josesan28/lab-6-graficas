use raylib::prelude::*;
const SIZE: usize = 256;
pub struct Surface<const N: usize> {
    values: Vec<[f32; N]>,
}
impl<const N: usize> Surface<N> {
    pub fn new(generate: impl Fn(Vector3) -> [f32; N] + Sync) -> Self {
        let mut values = vec![[0.0; N]; 6 * SIZE * SIZE];
        std::thread::scope(|scope| {
            for (face, data) in values.chunks_mut(SIZE * SIZE).enumerate() {
                let generate = &generate;
                scope.spawn(move || {
                    for (i, value) in data.iter_mut().enumerate() {
                        let u = (i % SIZE) as f32 / (SIZE - 1) as f32 * 2.0 - 1.0;
                        let v = (i / SIZE) as f32 / (SIZE - 1) as f32 * 2.0 - 1.0;
                        let p = match face {
                            0 => Vector3::new(1.0, v, u),
                            1 => Vector3::new(-1.0, v, u),
                            2 => Vector3::new(u, 1.0, v),
                            3 => Vector3::new(u, -1.0, v),
                            4 => Vector3::new(u, v, 1.0),
                            _ => Vector3::new(u, v, -1.0),
                        };
                        *value = generate(p.normalized());
                    }
                });
            }
        });
        Self { values }
    }
    pub fn sample(&self, p: Vector3) -> [f32; N] {
        let a = Vector3::new(p.x.abs(), p.y.abs(), p.z.abs());
        let (face, u, v) = if a.x >= a.y && a.x >= a.z {
            (if p.x >= 0.0 { 0 } else { 1 }, p.z / a.x, p.y / a.x)
        } else if a.y >= a.z {
            (if p.y >= 0.0 { 2 } else { 3 }, p.x / a.y, p.z / a.y)
        } else {
            (if p.z >= 0.0 { 4 } else { 5 }, p.x / a.z, p.y / a.z)
        };
        let x = ((u + 1.0) * 0.5 * (SIZE - 1) as f32).clamp(0.0, (SIZE - 1) as f32);
        let y = ((v + 1.0) * 0.5 * (SIZE - 1) as f32).clamp(0.0, (SIZE - 1) as f32);
        let ix = (x as usize).min(SIZE - 2);
        let iy = (y as usize).min(SIZE - 2);
        let tx = x - ix as f32;
        let ty = y - iy as f32;
        let i = face * SIZE * SIZE + iy * SIZE + ix;
        std::array::from_fn(|n| {
            let top = self.values[i][n] * (1.0 - tx) + self.values[i + 1][n] * tx;
            let bottom = self.values[i + SIZE][n] * (1.0 - tx) + self.values[i + SIZE + 1][n] * tx;
            top * (1.0 - ty) + bottom * ty
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interpolation_is_continuous_across_cube_edges_and_corners() {
        let map = Surface::new(|p| [p.x, p.y, p.z]);
        for axis in [-1.0, 1.0] {
            for step in -10..=10 {
                for epsilon in [-0.0001, 0.0001] {
                    let p = Vector3::new(axis, axis + epsilon, step as f32 / 10.0);
                    let expected = p.normalized();
                    let actual = map.sample(p);
                    assert!(
                        (Vector3::new(actual[0], actual[1], actual[2]) - expected).length()
                            < 0.0001
                    );
                }
            }
        }
    }
}
