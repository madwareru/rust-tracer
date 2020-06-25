mod color;
mod picture;
mod ray;

use picture::Picture;
use ray::Ray;

use cgmath::{Vector3, VectorSpace, InnerSpace};

const WHITE_COLOR: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
const SKY_COLOR: Vector3<f32> = Vector3::new(0.35, 0.375, 0.675);

fn test_sphere(ray: &Ray, sphere: (Vector3<f32>, f32)) -> Option<(Vector3<f32>, Vector3<f32>)> {
    // (ro + rd * t - sc)^2 = r^2
    // so = ro -sc
    // dot(so + rd * t, so + rd * t) = r*r
    // dot(so, so) + t*t * dot(rd, rd) + 2*t*dot(so, rd) - r*r = 0
    // a = dot(dr, rd)
    // b = 2 * dot(so, rd)
    // c = dot(so, so) - r*r
    let (center, radius) = sphere;
    let oc = ray.origin - center;
    let a = ray.direction.dot(ray.direction);
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        None
    } else {
        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            None
        } else {
            let p = ray.get_point_at(t);
            let n = (p - center) / radius;
            Some((p, n))
        }
    }
}

fn sky_color(ray: &Ray) -> Vector3<f32> {
    let y_remapped = (ray.direction.y + 1.0) / 2.0;
    WHITE_COLOR.lerp(SKY_COLOR, y_remapped)
}

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        let height_multiplier = 2.0;
        let height_corr = height_multiplier / 2.0;
        let width_multiplier = w as f32 / h as f32 * height_multiplier;
        let width_corr = width_multiplier / 2.0;
        for j in 0..h {
            for i in 0..w {
                let dir = Vector3::new(
                    i as f32 / w as f32 * width_multiplier - width_corr,
                    j as f32 / h as f32 * height_multiplier - height_corr,
                    -1.0
                );
                let ray = Ray{
                    origin: Vector3::new(0.0, 0.0, 0.0),
                    direction: dir.normalize()
                };
                let sky_color = sky_color(&ray);
                let final_color = if let Some((p, n)) = test_sphere(&ray, (Vector3::new(0.0, 0.0, -1.0), 0.5)) {
                    Vector3::new(n.x + 1.0, n.y + 1.0, n.z + 1.0) * 0.5
                } else {
                    sky_color
                };
                colors[i + j * w] = (final_color * 255.99).into();
            }
        }
    });
    pic.print_as_ppm();
}
