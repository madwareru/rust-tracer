use cgmath::{Vector3, vec3};
use crate::shape::Shape;
use crate::ray::Ray;

#[derive(Clone, Copy)]
pub struct AaBb {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>
}

const EPS: f32 = 0.001;

impl<'a> From<Shape<'a>> for Option<AaBb> {
    fn from(shape: Shape<'a>) -> Self {
        match shape {
            Shape::TriangleMesh { center, mesh, material } => {
                let (mut min, mut max) = (vec3(0.0, 0.0, 0.0), (vec3(0.0, 0.0, 0.0)));
                for ix in 0..mesh.vertices.len() {
                    let p = mesh.vertices[ix].position + center;
                    if p.x < min.x {min.x = p.x}
                    if p.x > max.x {max.x = p.x}
                    if p.y < min.y {min.y = p.y}
                    if p.y > max.y {max.y = p.y}
                    if p.z < min.z {min.z = p.z}
                    if p.z > max.z {max.z = p.z}
                }
                min -= vec3(EPS, EPS, EPS);
                max += vec3(EPS, EPS, EPS);
                Some(AaBb{min, max})
            },
            _ => {
                // Shapes different from TriangleMesh considered to be relatively equivalent to AaBb
                // in the sense of difficulty to calculate hit so it does not provide AaBb
                None
            },
        }
    }
}

impl AaBb {
    pub fn is_hit(&self, ray: &Ray) -> bool {
        let (mut t_min, mut t_max) = (0.00001, 10000.0);
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction[a];
            let (t0, t1) = if inv_d < 0.0 {
                (
                    (self.max[a] - ray.origin[a]) * inv_d,
                    (self.min[a] - ray.origin[a]) * inv_d
                )
            } else {
                (
                    (self.min[a] - ray.origin[a]) * inv_d,
                    (self.max[a] - ray.origin[a]) * inv_d
                )
            };
            if t0 > t_min { t_min = t0}
            if t1 < t_max { t_max = t1}
            if t_max < t_min {
                return false;
            }
        }
        true
    }
}