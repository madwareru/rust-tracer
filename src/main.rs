mod color;
mod picture;
mod ray;
mod shape;
mod world;
mod material;

use picture::Picture;
use ray::Ray;

use cgmath::{Vector3, VectorSpace, InnerSpace};
use crate::ray::{HitInfo};
use crate::shape::Shape;
use rand::prelude::*;
use crate::world::World;
use crate::material::{Material, MaterialDetails};

const SUN_VECTOR: Vector3<f32> = Vector3::new(0.7, 0.7, -0.5);
const WHITE_COLOR: Vector3<f32> = Vector3::new(0.9, 0.95, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35, 0.575, 0.875);
const NUM_SAMPLES: u16 = 256;

const LIGHT_GRAY_MAT: Material = Material {
    color: Vector3::new(0.8, 0.8, 0.8),
    details: MaterialDetails::Lambertian
};

const RED_MAT: Material = Material {
    color: Vector3::new(1.0, 0.25, 0.25),
    details: MaterialDetails::Lambertian
};

const GREEN_MAT: Material = Material {
    color: Vector3::new(0.1, 0.3, 0.1),
    details: MaterialDetails::Lambertian
};

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let sun_vector = SUN_VECTOR.normalize();
    let sunny = 1.0 - (sun_vector.dot(ray.direction).max(0.0).powf(16.0));
    WHITE_COLOR.lerp(SKY_COLOR, sunny)
}

fn gamma_correct(v: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: v.x.sqrt(),
        y: v.y.sqrt(),
        z: v.z.sqrt()
    }
}

fn mul(l: Vector3<f32>, r: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: l.x * r.x,
        y: l.y * r.y,
        z: l.z * r.z
    }
}

fn sample_color<'a>(ray: &'a Ray, world: &'a World, rng: &'a mut ThreadRng) -> Vector3<f32> {
    let nearest_hit = ray.hit_test(world);
    if let Some(HitInfo{n: normal, p: point, material, ..}) = nearest_hit {
        if let Some((clr, ray_reflect)) = material.scatter(rng, &nearest_hit.unwrap()) {
            mul(sample_color(&ray_reflect, world, rng), clr)
        } else {
            sky_color(&ray)
        }
    } else {
        sky_color(&ray)
    }
}

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        let height_multiplier = 2.0;
        let height_corr = height_multiplier / 2.0;
        let width_multiplier = w as f32 / h as f32 * height_multiplier;
        let width_corr = width_multiplier / 2.0;
        let mut rng = rand::thread_rng();

        let world = World{ shapes: &[
            Shape::Sphere{
                center: Vector3::new(0.0, 0.0, -1.0),
                radius: 0.5,
                material: RED_MAT
            },
            Shape::Sphere{
                center: Vector3::new(0.25, -0.4, -0.65),
                radius: 0.1,
                material: GREEN_MAT
            },
            Shape::Sphere{
                center: Vector3::new(0.0, -100.0, -1.0),
                radius: 99.5,
                material: LIGHT_GRAY_MAT
            }
        ]};

        for j in 0..h {
            for i in 0..w {
                let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                for _ in 0..NUM_SAMPLES {
                    let dir = Vector3::new(
                        (i as f32 + rng.gen::<f32>()) / w as f32 * width_multiplier - width_corr,
                        (j as f32 + rng.gen::<f32>()) / h as f32 * height_multiplier - height_corr,
                        -1.0
                    );

                    let ray = Ray{
                        origin: Vector3::new(0.0, 0.0, 0.0),
                        direction: dir.normalize()
                    };
                    pixel_color += sample_color(&ray, &world, &mut rng);
                }
                pixel_color /= NUM_SAMPLES as f32;
                colors[i + j * w] = (gamma_correct(pixel_color) * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}
