#[macro_use]
extern crate lazy_static;

mod color;
mod picture;
mod ray;
mod shape;
mod world;
mod material;
mod camera;
mod vector_utils;

use {
    cgmath::{Vector3, VectorSpace, InnerSpace, vec3},
    rayon::{prelude::*},
    std::env,
    rand::prelude::*,
    picture::*,
    ray::*,
    shape::*,
    world::*,
    material::*,
    camera::{Camera, Origin, Up, Fov, Target}
};
use cgmath::Quaternion;

const SUN_VECTOR: Vector3<f32> = Vector3::new(0.7, 0.7, -0.5);
const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35 / 14.0, 0.575 / 14.0, 0.875 / 14.0);
const NUM_SAMPLES: u16 = 512;
const FOCUS_DISTANCE: f32 = 1.9;
const APERTURE: f32 = 0.05;
const MAX_T: f32 = 400.0;

const LIGHT_GRAY_MAT: Material = Material {
    albedo: Vector3::new(0.8, 0.8, 0.8),
    details: MaterialDetails::Metallic {roughness: 0.2},
    emittance: 0.0
};

const DARK_GRAY_MAT: Material = Material {
    albedo: Vector3::new(0.1, 0.1, 0.1),
    details: MaterialDetails::Metallic {roughness: 0.6},
    emittance: 0.0
};

const BLUE_MAT: Material = Material {
    albedo: Vector3::new(0.1, 0.1, 1.0),
    details: MaterialDetails::Dielectric { ref_idx: 1.1, roughness: 0.1 },
    emittance: 0.0
};

const LIGHT_GRAY_MAT_LAMBERT: Material = Material {
    albedo: Vector3::new(0.8, 0.8, 0.8),
    details: MaterialDetails::Lambertian,
    emittance: 0.0
};

const WHITE_BULB_MAT: Material = Material {
    albedo: Vector3::new(3.0, 3.0, 3.0),
    details: MaterialDetails::Lambertian,
    emittance: 1.0
};

const DIELECTRIC_MAT: Material = Material {
    albedo: Vector3::new(0.0, 1.0, 0.4),
    details: MaterialDetails::Dielectric {
        ref_idx: 1.5,
        roughness: 0.0
    },
    emittance: 0.0
};

const RED_MAT: Material = Material {
    albedo: Vector3::new(1.0, 0.25, 0.25),
    details: MaterialDetails::Metallic {roughness: 0.0},
    emittance: 0.0
};

const ORANGE_MAT: Material = Material {
    albedo: Vector3::new(1.0, 0.4, 0.0),
    details: MaterialDetails::Lambertian,
    emittance: 0.9
};

const WORLD: World =
    World{ shapes: &[
        Shape::Disk{
            center: Vector3::new(-0.85, 0.49, 1.05),
            radius: 0.125,
            normal: vec3(0.0, -1.0, 0.0),
            material: WHITE_BULB_MAT
        },
        Shape::Disk{
            center: Vector3::new(0.85, 0.49, 1.05),
            radius: 0.125 / 2.0,
            normal: vec3(0.0, -1.0, 0.0),
            material: WHITE_BULB_MAT
        },
        Shape::Disk{
            center: Vector3::new(0.0, 0.49, -1.05),
            radius: 0.125 / 4.0,
            normal: vec3(0.0, -1.0, 0.0),
            material: WHITE_BULB_MAT
        },
        Shape::Sphere{
            center: Vector3::new(-0.6, -0.3, 0.7),
            radius: 0.15,
            material: DIELECTRIC_MAT
        },
        Shape::Sphere{
            center: Vector3::new(0.0, 0.0, 1.0),
            radius: 0.5,
            material: RED_MAT
        },
        Shape::Sphere{
            center: Vector3::new(0.25, -0.4, 0.65),
            radius: 0.1,
            material: ORANGE_MAT
        },
        Shape::Cube {
            center: Vector3::new(-0.25, -0.4, 0.35),
            sizes: vec3(0.2, 0.2, 0.2),
            rotation: Quaternion::new(0.5, 0.0, 1.0, 0.0),
            material: BLUE_MAT
        },
        Shape::Sphere{
            center: Vector3::new(0.15, -0.45, 0.55),
            radius: 0.05,
            material: DARK_GRAY_MAT
        },
        Shape::Sphere{
            center: Vector3::new(-0.75, -0.45, 0.75),
            radius: 0.05,
            material: DARK_GRAY_MAT
        },
        Shape::Disk{
            center: Vector3::new(0.0, -0.5, 1.0),
            radius: 2.0,
            normal: vec3(0.0, 1.0, 0.0),
            material: LIGHT_GRAY_MAT_LAMBERT
        },
        Shape::Sphere{
            center: Vector3::new(0.0, 100.0, 1.0),
            radius: 99.5,
            material: LIGHT_GRAY_MAT
        }
    ]};

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

fn sample_color<'a>(ray: &'a Ray, world: &'a World, rng: &'a mut ThreadRng, depth: u8) -> Vector3<f32> {
    let nearest_hit = ray.hit_test(world);
    if depth == 0 {
        return Vector3::new(0.0, 0.0, 0.0);
    }
    let sky_clr = sky_color(&ray);
    if let Some(HitInfo{material, t, ..}) = nearest_hit {
        if let Some((clr, ray_reflect)) = material.scatter(&ray, rng, &nearest_hit.unwrap()) {
            let c = mul(sample_color(&ray_reflect, world, rng, depth-1), clr).lerp(material.albedo, material.emittance);
            if t > MAX_T {
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

fn render_sample(
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
        forward_vector).normalize() * FOCUS_DISTANCE;

    let offset_disk = vector_utils::get_random_in_unit_disk(&mut rng) * APERTURE;
    let origin_with_offset =
        camera.origin +
        right_vector * offset_disk.x +
        up_vector * offset_disk.y;

    let ray = Ray{
        origin: origin_with_offset,
        direction: (camera.origin + dir - origin_with_offset).normalize()
    };

    sample_color(&ray, &WORLD, &mut rng, 10)
}

fn render_scene(times: u64) {
    let mut pic = Picture::new(640, 400);
    let t = (times + 100) as f32 / 50.0;
    pic.mutate(|colors, w, h| {
        let aspect = w as f32 / h as f32;
        let camera = Camera::new(
            Origin(Vector3::new(0.0, 0.0, 1.0) + Vector3::new(2.0 * t.cos(), 0.0, 2.0 * t.sin())),
            Up(Vector3::unit_y()),
            Fov(70.0f32.to_radians()),
            Target(Vector3::new(0.0, 0.0, 1.0))
        );
        let sequence = (0..NUM_SAMPLES).collect::<Vec<_>>();
        let basis_vectors = camera.get_basis_vectors(aspect);
        let mut stride = 0;
        let fact_samples = NUM_SAMPLES as f32;
        for j in 0..h {
            for i in 0..w {
                let pixel_color = sequence
                    .par_iter()
                    .cloned()
                    .map(|_| render_sample(i, j, w, h, camera, basis_vectors))
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

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        eprintln!("Usage: rust-tracer N > some.ppm");
        return;
    }
    let t: u64 = args.nth(1).unwrap().parse().unwrap();
    render_scene(t);
}
