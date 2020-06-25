use cgmath::Vector3;

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>
}

impl Ray {
    fn get_point_at(&self, t: f32) -> Vector3<f32> {
        self.origin + self.direction * t
    }
}