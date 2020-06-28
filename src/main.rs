mod color;
mod picture;
mod ray;
mod shape;
mod world;
mod material;
mod camera;

use {
    cgmath::{Vector3, VectorSpace, InnerSpace},
    rand::prelude::*,
    picture::*,
    ray::*,
    shape::*,
    world::*,
    material::*,
    camera::{Camera, Origin, Up, Fov, Target}
};

const SUN_VECTOR: Vector3<f32> = Vector3::new(0.7, 0.7, -0.5);
const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35/14.0, 0.575/14.0, 0.875/14.0);
const NUM_SAMPLES: u16 = 512;

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

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let sun_vector = SUN_VECTOR.normalize();
    let sunny = 1.0 - (sun_vector.dot(ray.direction).max(0.0).powf(16.0));
    WHITE_COLOR.lerp(SKY_COLOR, sunny)
}

fn gamma_correct(v: Vector3<f32>) -> Vector3<f32> {
    Vector3 {
        x: v.x.min(1.0).sqrt(),
        y: v.y.min(1.0).sqrt(),
        z: v.z.min(1.0).sqrt()
    }
}

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

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        let mut rng = rand::thread_rng();
        let aspect = w as f32 / h as f32;
        let camera = Camera::new(
            Origin(Vector3::new(0.0, 0.0, -1.0)),
            Up(Vector3::unit_y()),
            Fov(70.0f32.to_radians()),
            Target(Vector3::new(0.0, 0.0, 1.0))
        );
        let (right_vector, up_vector, forward_vector) =
            camera.get_basis_vectors(aspect);

        let height_multiplier = 2.0;
        let height_corr = height_multiplier / 2.0;
        let width_multiplier = aspect * height_multiplier;
        let width_corr = width_multiplier / 2.0;

        let world = World{ shapes: &[
            Shape::Sphere{
                center: Vector3::new(-0.85, -0.0, 1.05),
                radius: 0.25,
                material: WHITE_BULB_MAT
            },
            Shape::Sphere{
                center: Vector3::new(-0.6, -0.2, 0.7),
                radius: 0.15,
                material: DARK_GRAY_MAT
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

        for j in 0..h {
            for i in 0..w {
                let mut pixel_color = Vector3::new(0.0, 0.0, 0.0);
                for _ in 0..NUM_SAMPLES {
                    let i = (i as f32 + rng.gen::<f32>() - 0.5) / w as f32 * width_multiplier - width_corr;
                    let j = ((j as f32 + rng.gen::<f32>() - 0.5) / h as f32 * height_multiplier - height_corr);
                    let dir =
                        right_vector * i +
                        up_vector * j +
                        forward_vector;

                    let ray = Ray{
                        origin: camera.origin,
                        direction: dir.normalize()
                    };

                    pixel_color += sample_color(&ray, &world, &mut rng, 10);
                }
                pixel_color /= NUM_SAMPLES as f32;
                colors[i + j * w] = (gamma_correct(pixel_color) * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}
