use {
    cgmath::Vector3,
    crate::material::Material
};

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vector3<f32>,
    pub direction: Vector3<f32>
}

#[derive(Clone, Copy)]
pub struct HitInfo {
    pub t: f32,
    pub p: Vector3<f32>,
    pub n: Vector3<f32>,
    pub material: Material
}

pub trait HitTestable {
    fn hit_test(&self, ray: &Ray) -> Option<HitInfo>;
}

impl Ray {
    pub fn get_point_at(&self, t: f32) -> Vector3<f32> {
        self.origin + self.direction * t
    }
    pub fn hit_test<T: HitTestable>(&self, hit_testable: &T) -> Option<HitInfo> {
        hit_testable.hit_test(self)
    }
}