use {
    cgmath::{Vector2, Vector3, InnerSpace, VectorSpace, vec2, vec3},
    crate::ray::{HitInfo, Ray},
    crate::vector_utils::*,
    rand:: {
        prelude::ThreadRng,
        Rng
    }
};

pub trait AlbedoFn {
    fn get_color(&self, uv: Vector2<f32>) -> Vector3<f32>;
}

#[derive(Copy, Clone)]
pub enum Albedo<'a> {
    Constant(Vector3<f32>),
    Checker(f32),
    Texture(usize, usize, &'a[Vector3<f32>]),
}

impl AlbedoFn for Albedo<'_> {
    fn get_color(&self, uv: Vector2<f32>) -> Vector3<f32> {
        match self {
            Albedo::Constant(color) => *color,
            Albedo::Checker(scale) => {
                let uv_scale = uv * *scale;
                let x_even = uv_scale.x as u32 & 1 == 0;
                let y_even = uv_scale.y as u32 & 1 == 0;
                if (x_even && !y_even) || (y_even && !x_even) {
                    vec3(0.0, 0.0, 0.0)
                } else {
                    vec3(1.0, 1.0, 1.0)
                }
            },
            Albedo::Texture(w, h, pixels) => {
                let (u, v) = (uv.x as usize, uv.y as usize);
                let (h_t, v_t) = (uv.x - u as f32, uv.y - v as f32);

                let next_u = (u + 1).min(*w);
                let next_v = (v + 1).min(*h);
                let px0_idx = u + v * *w;
                let px1_idx = next_u + v * *w;
                let px2_idx = u + next_v * *w;
                let px3_idx = next_u + next_v * *w;
                let p0 = pixels[px0_idx].lerp(pixels[px1_idx], h_t);
                let p1 = pixels[px2_idx].lerp(pixels[px3_idx], h_t);
                p0.lerp(p1, v_t)
            },
        }
    }
}

#[derive(Copy, Clone)]
pub enum MaterialDetails {
    Lambertian,
    Metallic{ roughness: f32 },
    Dielectric{ ref_idx: f32, roughness: f32 },
}

#[derive(Copy, Clone)]
pub struct Material<'a> {
    pub albedo: Albedo<'a>,
    pub emittance: f32,
    pub details: MaterialDetails
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

impl Material<'_> {
    pub fn scatter(&self, ray_in:&Ray, rng: &mut ThreadRng, hit: &HitInfo)
        -> Option<(Vector3<f32>, Ray)>
    {
        let &HitInfo{n: normal, p: point, uv, ..} = hit;
        let uv = uv.unwrap_or(vec2(0.0, 0.0));
        let albedo = self.albedo.get_color(uv);
        match self.details {
            MaterialDetails::Lambertian => {
                let target = point + normal + get_random_in_unit_sphere(rng);
                let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                Some((albedo, ray_reflect))
            },
            MaterialDetails::Metallic { roughness } => {
                let reflected_ray_dir = reflect(&ray_in.direction, &normal);
                if reflected_ray_dir.dot(normal) > 0.0 {
                    let target = point + reflected_ray_dir + get_random_in_unit_sphere(rng) * roughness;
                    let ray_reflect = Ray{origin : point, direction: (target - point).normalize()};
                    Some((albedo, ray_reflect))
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
                        Some((albedo, ray_reflect))
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
                        Some((albedo, ray_scattered))
                    },
                }
            }
        }
    }
}