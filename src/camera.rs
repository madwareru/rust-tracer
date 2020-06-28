use {
    cgmath::{Vector3, VectorSpace, InnerSpace}
};

pub struct Origin(pub Vector3<f32>);
pub struct Up(pub Vector3<f32>);
pub struct Target(pub Vector3<f32>);
pub struct Fov(pub f32);

#[derive(Clone, Copy)]
pub struct Camera {
    pub origin: Vector3<f32>,
    pub fov: f32,
    pub up: Vector3<f32>,
    pub direction: Vector3<f32>
}

impl Camera {
    pub fn new(origin: Origin, up: Up, fov: Fov, target: Target) -> Self {
        Camera {
            origin: origin.0,
            up: up.0,
            fov: fov.0,
            direction: (target.0 - origin.0).normalize()
        }
    }

    pub fn orient_at(&mut self, target: Vector3<f32>) {
        self.direction = (target - self.origin).normalize();
    }

    pub fn get_basis_vectors(&self, aspect: f32) -> (Vector3<f32>, Vector3<f32>, Vector3<f32>) {
        let forward = self.direction.normalize();
        let right = self.up.cross(forward);
        let up = forward.cross(right);
        (right, up, forward * aspect / (self.fov / 2.0).tan())
    }

}

