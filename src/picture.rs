use crate::color::Color;

pub struct Picture {
    width: usize,
    height: usize,
    colors: Vec<Color>
}

impl Picture {
    pub fn new(width: usize, height: usize) -> Self {
        Picture {
            width,
            height,
            colors: vec![Color{r: 0, g: 0, b: 0}; width * height]
        }
    }
    pub fn mutate<'a, F>(&'a mut self, mutator: F)
        where F: FnOnce(&'a mut[Color], usize, usize) -> ()
    {
        mutator(&mut self.colors, self.width, self.height);
    }
    pub fn print_as_ppm(&self) {
        println!("P3");
        println!("{} {}", self.width, self.height);
        println!("255");
        for j in (0..self.height).rev() {
            for i in 0..self.width {
                print!("{}", self.colors[j * self.width + i].to_string())
            }
            println!()
        }
    }

}