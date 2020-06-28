use {
    cgmath::{Vector3, InnerSpace},
    crate::ray::{HitInfo, Ray},
    rand:: {
        prelude::ThreadRng,
        Rng
    }
};

#[derive(Copy, Clone)]
pub enum MaterialDetails {
    Lambertian,
    Metallic{ roughness: f32 }
}

#[derive(Copy, Clone)]
pub struct Material {
    pub albedo: Vector3<f32>,
    pub emittance: f32,
    pub details: MaterialDetails
}

fn get_random_in_unit_sphere(rng: &mut ThreadRng) -> Vector3<f32> {
    Vector3::new(
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5,
        rng.gen::<f32>() - 0.5
    ).normalize()
}

fn reflect(v: &Vector3<f32>, n: &Vector3<f32>) -> Vector3<f32> {
    v - n * 2.0 * v.dot(*n)
}

impl Material {
    pub fn scatter(&self, ray_in:&Ray, rng: &mut ThreadRng, hit: &HitInfo)
        -> Option<(Vector3<f32>, Ray)>
    {
        let &HitInfo{n: normal, p: point, ..} = hit;
        match self.details {
            MaterialDetails::Lambertian => {
                let target = point + normal + get_random_in_unit_sphere(rng);
                let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                Some((self.albedo, ray_reflect))
            },
            MaterialDetails::Metallic { roughness } => {
                let reflected_ray_dir = reflect(&ray_in.direction, &normal);
                if reflected_ray_dir.dot(normal) > 0.0 {
                    let target = point + reflected_ray_dir + get_random_in_unit_sphere(rng) * roughness;
                    let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                    Some((self.albedo, ray_reflect))
                } else {
                    None
                }
            }
        }
    }
}