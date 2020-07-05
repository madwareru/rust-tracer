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
    std::env,
    rand::prelude::*,
    picture::*,
    ray::*,
    shape::*,
    world::*,
    material::*,
    color::Color,
    camera::{Camera, Origin, Up, Fov, Target}
};

const SUN_VECTOR: Vector3<f32> = Vector3::new(0.7, 0.7, -0.5);
const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35/14.0, 0.575/14.0, 0.875/14.0);
const NUM_SAMPLES: u16 = 1024;
const FOCUS_DISTANCE: f32 = 1.9;
const APERTURE: f32 = 0.05;

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

const LIGHT_GRAY_MAT_LAMBERT: Material = Material {
    albedo: Vector3::new(0.8, 0.8, 0.8),
    details: MaterialDetails::Lambertian,
    emittance: 0.0
};

const WHITE_BULB_MAT: Material = Material {
    albedo: Vector3::new(1.0, 1.0, 1.0),
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
        Shape::Sphere{
            center: Vector3::new(-0.85, -0.0, 1.05),
            radius: 0.25,
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
        Shape::Sphere{
            center: Vector3::new(-0.25, -0.45, 0.65),
            radius: 0.05,
            material: DARK_GRAY_MAT
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
        Shape::Sphere{
            center: Vector3::new(0.0, -100.0, 1.0),
            radius: 99.5,
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
    if let Some(HitInfo{material, ..}) = nearest_hit {
        if let Some((clr, ray_reflect)) = material.scatter(&ray, rng, &nearest_hit.unwrap()) {
            mul(sample_color(&ray_reflect, world, rng, depth-1), clr).lerp(material.albedo, material.emittance)
        } else {
            Vector3::new(0.0, 0.0, 0.0)
        }
    } else {
        sky_color(&ray)
    }
}

fn render_sample(i: usize,
                 j: usize,
                 w: usize,
                 h: usize,
                 camera: &Camera,
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
    let mut pic = Picture::new(1280, 800);
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
        for j in 0..h {
            for i in 0..w {
                let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                for _ in 0..NUM_SAMPLES {
                    pixel_color += render_sample(i, j, w, h, &camera, basis_vectors);
                }
                pixel_color /= NUM_SAMPLES as f32;
                colors[i + j * w] = (gamma_correct(pixel_color) * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}

fn render_disc(times: u64) {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        let mut rng = rand::thread_rng();
        for _ in 0..times {
            let disk = vector_utils::get_random_in_unit_disk(&mut rng);
            let disk = disk * 100.0 +
                vec3(160.0, 100.0, 0.0);
            let x = (disk.x - 0.5) as i32;
            let y = (disk.y - 0.5) as i32;
            for j in 0..2 {
                for i in 0..2 {
                    let ox = x + i;
                    let oy = y + j;
                    if (ox < w as i32) && (ox >= 0) && (oy < h as i32) && (oy >= 0) {
                        let idx = ox as usize + oy as usize * w;
                        colors[idx] = Color { r: 255, g: 255, b: 255};
                    }
                }
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
    //render_disc(t);
    render_scene(t);
}
