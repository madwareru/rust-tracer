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
    Metallic{ roughness: f32 },
    Dielectric{ ref_idx: f32, roughness: f32 },
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

fn refract(v: &Vector3<f32>, n: &Vector3<f32>, ni_over_nt: f32) -> Option<Vector3<f32>> {
    let uv = v.normalize();
    let dt = uv.dot(*n);
    let discriminant = 1.0 - ni_over_nt*ni_over_nt*(1.0 - dt*dt);
    if discriminant > 0.0 {
        Some((uv - n*dt) * ni_over_nt - *n * discriminant.sqrt())
    } else {
        None
    }
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
            MaterialDetails::Dielectric { ref_idx, roughness } => {
                let reflected_ray_dir = reflect(&ray_in.direction, &normal);
                let cosine = ray_in.direction.dot(hit.n);
                let (outward_n, ni_over_nt, cosine) = if ray_in.direction.dot(hit.n) > 0.0 {
                    (-hit.n, ref_idx, cosine)
                } else {
                    (hit.n, 1.0 / ref_idx, -cosine)
                };
                match refract(&ray_in.direction, &outward_n, ni_over_nt) {
                    None => {
                        let target = point + reflected_ray_dir + get_random_in_unit_sphere(rng) * roughness;
                        let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                        Some((self.albedo, ray_reflect))
                    },
                    Some(refracted_ray_dir) => {
                        let scattered_dir = {
                            let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
                            let r0 = r0 * r0;
                            let reflect_probability = r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0);
                            if rng.gen::<f32>() < reflect_probability {
                                reflected_ray_dir
                            } else {
                                refracted_ray_dir
                            }
                        };
                        let target = point + scattered_dir + get_random_in_unit_sphere(rng) * roughness;
                        let ray_scattered = Ray{
                            origin : point,
                            direction: (target - point).normalize()
                        };
                        Some((self.albedo, ray_scattered))
                    },
                }
            }
        }
    }
}