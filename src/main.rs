mod color;
mod picture;

use color::Color;
use picture::Picture;

fn main() {
    let mut pic = Picture::new(320, 200);
    pic.mutate(|colors, w, h| {
        for j in 0..h {
            let j_float = j as f32 / h as f32 * 255.99;
            for i in 0..w {
                let i_float = i as f32 / w as f32 * 255.99;
                let clr = Color{r: i_float as u8, g: j_float as u8, b: 127};
                colors[i + j * w] = clr;
            }
        }
    });
    pic.print_as_ppm();
}
