use cgmath::Vector3;

#[derive(Clone, Copy)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn to_string(&self) -> String {
        format!("{}\t{}\t{}\t", self.r, self.g, self.b)
    }
}

impl From<Vector3<f32>> for Color {
    fn from(v: Vector3<f32>) -> Self {
        Color {
            r: v.x as u8,
            g: v.y as u8,
            b: v.z as u8
        }
    }
}
