mod color;
mod picture;
mod ray;

use picture::Picture;
use ray::Ray;

use cgmath::{Vector3, VectorSpace, InnerSpace};

const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35, 0.375, 0.675);

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let dir_normalized = ray.direction.normalize();
    let y_remapped = (dir_normalized.y + 1.0) / 2.0;
    WHITE_COLOR.lerp(SKY_COLOR, y_remapped)
}

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        for j in 0..h {
            for i in 0..w {
                let dir = Vector3::new(
                    i as f32 / w as f32 * 4.0 - 2.0,
                    j as f32 / h as f32 * 2.0 - 1.0,
                    1.0
                );
                let ray = Ray{
                    origin: Vector3::new(0.0, 0.0, 0.0),
                    direction: dir
                };
                let sky_color = sky_color(&ray);
                let final_color = sky_color;
                colors[i + j * w] = (final_color * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}
