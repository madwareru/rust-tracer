use crate::ray::{HitInfo, Ray};
use cgmath::{Vector3, InnerSpace};
use rand::prelude::ThreadRng;
use rand::Rng;

#[derive(Copy, Clone)]
pub enum MaterialDetails {
    Lambertian,
    Metallic{ roughness: f32 }
}

#[derive(Copy, Clone)]
pub struct Material {
    pub color: Vector3<f32>,
    pub details: MaterialDetails
}

fn get_random_in_unit_sphere(rng: &mut ThreadRng) -> Vector3<f32> {
    Vector3::new(
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5
    ).normalize()
}

impl Material {
    pub fn scatter(&self, rng: &mut ThreadRng, hit: &HitInfo)
        -> Option<(Vector3<f32>, Ray)>
    {
        match self.details {
            MaterialDetails::Lambertian => {
                let &HitInfo{n: normal, p: point, ..} = hit;
                let target = point + normal + get_random_in_unit_sphere(rng);
                let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                Some((self.color, ray_reflect))
            },
            MaterialDetails::Metallic { .. } => {
                None
            },
        }
    }
}