mod color;
mod picture;

use picture::Picture;
use cgmath::Vector3;

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        for j in 0..h {
            for i in 0..w {
                let vec = Vector3::new(
                    i as f32 / w as f32 * 255.99,
                    j as f32 / h as f32 * 255.99,
                    127.0
                );
                colors[i + j * w] = vec.into();
            }
        }
    });
    pic.print_as_ppm();
}
