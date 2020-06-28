use{
    cgmath::{Vector3, InnerSpace},
    crate::ray::{HitTestable, HitInfo, Ray},
    crate::material::Material
};

#[derive(Copy, Clone)]
pub enum Shape {
    Sphere {
        center: Vector3<f32>,
        radius: f32,
        material: Material
    }
}

impl HitTestable for Shape {
    fn hit_test(&self, ray: &Ray) -> Option<HitInfo> {
        match self {
            Shape::Sphere { center, radius, material } => {
                let radius = *radius;
                let material = *material;
                let oc = ray.origin - center;
                let a = ray.direction.dot(ray.direction);
                let b = 2.0 * oc.dot(ray.direction);
                let c = oc.dot(oc) - radius * radius;
                let discriminant = b * b - 4.0 * a * c;
                if discriminant < 0.0 {
                    None
                } else {
                    let t = (-b - discriminant.sqrt()) / (2.0 * a);
                    if t < 0.0 {
                        None
                    } else {
                        let p = ray.get_point_at(t);
                        let n = (p - center) / radius;
                        Some(HitInfo{ t, p, n, material })
                    }
                }
            },
        }
    }
}