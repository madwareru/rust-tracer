use {
    crate::ray::{Ray, HitInfo},
    cgmath::{Vector3, vec3, vec2, InnerSpace, VectorSpace},
    rayon::prelude::*,
    crate::camera::{Camera, Origin, Up, Fov, Target},
    crate::picture::Picture,
    crate::vector_utils,
    crate::material::AlbedoFn,
    crate::world::World,
    rand::prelude::*
};

const SUN_VECTOR: Vector3<f32> = Vector3::new(0.7, 0.7, -0.5);
const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35 / 14.0, 0.575 / 14.0, 0.875 / 14.0);

pub struct Scene<'a> {
    pub world: World<'a>,
    pub focus_distance: f32,
    pub aperture: f32,
    pub num_samples: u16,
    pub max_t: f32
}

impl Scene<'_> {
    pub fn sample_color(&self, ray: &Ray, rng: &mut ThreadRng, depth: u8) -> Vector3<f32> {
        let nearest_hit = ray.hit_test(&self.world);
        if depth == 0 {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        let sky_clr = sky_color(&ray);
        if let Some(HitInfo{material, t, uv, ..}) = nearest_hit {
            if let Some((clr, ray_reflect)) = material.scatter(&ray, rng, &nearest_hit.unwrap()) {
                let uv = uv.unwrap_or(vec2(0.0, 0.0));
                let albedo = material.albedo.get_color(uv);
                let c = mul(self.sample_color(&ray_reflect, rng, depth-1), clr).lerp(albedo, material.emittance);
                if t > self.max_t {
                    sky_clr
                } else {
                    c
                }
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            }
        } else {
            sky_clr
        }
    }

    pub fn render_sample(
        &self,
        i: usize,
        j: usize,
        w: usize,
        h: usize,
        camera: Camera,
        basis_vectors: (Vector3<f32>, Vector3<f32>, Vector3<f32>)
    ) -> Vector3<f32>
    {
        let (right_vector, up_vector, forward_vector) = basis_vectors;
        let mut rng = thread_rng();
        let i = (i as f32 + rng.gen::<f32>() - 0.5) / w as f32 * 2.0 - 1.0;
        let j = (j as f32 + rng.gen::<f32>() - 0.5) / h as f32 * 2.0 - 1.0;
        let dir =
            (right_vector * i +
            up_vector * j +
            forward_vector).normalize() * self.focus_distance;

        let offset_disk = vector_utils::get_random_in_unit_disk(&mut rng) * self.aperture;
        let origin_with_offset =
            camera.origin +
            right_vector * offset_disk.x +
            up_vector * offset_disk.y;

        let ray = Ray{
            origin: origin_with_offset,
            direction: (camera.origin + dir - origin_with_offset).normalize()
        };

        self.sample_color(&ray, &mut rng, 10)
    }

    pub fn render_as_ppm(&self, times: u64, w: usize, h: usize) {
        let mut pic = Picture::new(w, h);
        let t = (times + 100) as f32 / 50.0;
        pic.mutate(|colors, w, h| {
            let aspect = w as f32 / h as f32;
            let camera = Camera::new(
                Origin(Vector3::new(0.0, 0.0, 1.0) + Vector3::new(2.0 * t.cos(), 0.0, 2.0 * t.sin())),
                Up(Vector3::unit_y()),
                Fov(70.0f32.to_radians()),
                Target(Vector3::new(0.0, 0.0, 1.0))
            );
            let basis_vectors = camera.get_basis_vectors(aspect);
            let mut stride = 0;
            let fact_samples = self.num_samples as f32;
            for j in 0..h {
                for i in 0..w {
                    let pixel_color = (0..self.num_samples)
                        .into_par_iter()
                        .map(|_| self.render_sample(i, j, w, h, camera, basis_vectors))
                        .reduce(
                            || vec3(0.0, 0.0, 0.0),
                            |a, b| a + b
                        ) / fact_samples;

                    colors[stride] = (gamma_correct(pixel_color) * 255.99).into();
                    stride += 1;
                }
            }
        });
        pic.print_as_ppm();
    }
}

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let sun_vector = SUN_VECTOR.normalize();
    let sunny = 1.0 - (sun_vector.dot(ray.direction).max(0.0).powf(16.0));
    WHITE_COLOR.lerp(SKY_COLOR, sunny)
}

#[inline]
fn gamma_correct(v: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: v.x.min(1.0).sqrt(),
        y: v.y.min(1.0).sqrt(),
        z: v.z.min(1.0).sqrt()
    }
}

#[inline]
fn mul(l: Vector3<f32>, r: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: l.x * r.x,
        y: l.y * r.y,
        z: l.z * r.z
    }
}