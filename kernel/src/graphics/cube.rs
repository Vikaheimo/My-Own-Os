use libm::{cosf, sinf};
#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn rotate(&self, ax: f32, ay: f32) -> Self {
        let sin_x = sinf(ax);
        let cos_x = cosf(ax);
        let sin_y = sinf(ay);
        let cos_y = cosf(ay);

        // Rotate around X
        let y = self.y * cos_x - self.z * sin_x;
        let mut z = self.y * sin_x + self.z * cos_x;

        // Rotate around Y
        let x = self.x * cos_y + z * sin_y;
        z = -self.x * sin_y + z * cos_y;

        Vec3 { x, y, z }
    }
}

pub const CUBE_VERTICES: [Vec3; 8] = [
    Vec3::new(-1.0, -1.0, -1.0),
    Vec3::new(1.0, -1.0, -1.0),
    Vec3::new(1.0, 1.0, -1.0),
    Vec3::new(-1.0, 1.0, -1.0),
    Vec3::new(-1.0, -1.0, 1.0),
    Vec3::new(1.0, -1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
    Vec3::new(-1.0, 1.0, 1.0),
];

pub const EDGES: [(usize, usize); 12] = [
    (0, 1),
    (1, 2),
    (2, 3),
    (3, 0),
    (4, 5),
    (5, 6),
    (6, 7),
    (7, 4),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
];
