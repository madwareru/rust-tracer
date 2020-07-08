use std::io::Read;
use png::{Decoder, ColorType};
use cgmath::{Vector3, vec3};

pub struct ImgData {
    pub width: usize,
    pub height: usize,
    pub colors: Vec<Vector3<f32>>
}

pub fn load_png<R: Read>(r: R) -> ImgData {
    let mut decoder = Decoder::new(r);
    let (info, mut reader) = decoder.read_info().unwrap();
    let (w, h) = (info.width as usize, info.height as usize);
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();

    let bytes_per_pixel = match reader.output_color_type()
    {
        (ColorType::RGBA, _) => 4,
        (ColorType::RGB, _) => 3,
        (ColorType::Grayscale, _) => 1,
        (ColorType::GrayscaleAlpha, _) => 2,
        _ => panic!("unsupported color type")
    };
    let mut vec = vec![Vector3::new(0.0, 0.0, 0.0); w*h];
    let mut offset = 0;
    for i in 0..w*h {
        if bytes_per_pixel > 2 {
            vec[i] = vec3(
                buf[offset] as f32 / 255.0,
                buf[offset + 1] as f32 / 255.0,
                buf[offset + 2] as f32 / 255.0
            );
        } else {
            let luma = buf[offset] as f32 / 255.0;
            vec[i] = vec3(
                luma,
                luma,
                luma
            );
        }
        offset += bytes_per_pixel;
    }
    ImgData {width: w, height: h, colors: vec}
}