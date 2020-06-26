mod color;
mod picture;
mod ray;
mod shape;
mod world;

use picture::Picture;
use ray::Ray;

use cgmath::{Vector3, VectorSpace, InnerSpace};
use crate::ray::{HitInfo};
use crate::shape::Shape;
use rand::prelude::*;
use crate::world::World;

const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35, 0.575, 0.875);
const NUM_SAMPLES: u16 = 100;

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let y_remapped = (ray.direction.y + 1.0) / 2.0;
    WHITE_COLOR.lerp(SKY_COLOR, y_remapped)
}

fn gamma_correct(v: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: v.x.sqrt(),
        y: v.y.sqrt(),
        z: v.z.sqrt()
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
                radius: 0.5
            },
            Shape::Sphere{
                center: Vector3::new(0.0, -14.0, -1.0),
                radius: 13.5
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
                    let sky_color = sky_color(&ray);
                    let nearest_hit = ray.hit_test(&world);

                    pixel_color += if let Some(HitInfo{n: normal, ..}) = nearest_hit {
                        Vector3::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0) * 0.5
                    } else {
                        sky_color
                    };
                }
                pixel_color /= NUM_SAMPLES as f32;
                colors[i + j * w] = (gamma_correct(pixel_color) * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}
