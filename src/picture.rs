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
    pub fn mutate<'a, F>(&'a mut self, mut mutator: F)
        where F: FnMut(&'a mut[Color], usize, usize) -> ()
    {
        mutator(&mut self.colors, self.width, self.height);
    }
    pub fn print_to_file(self) {
        println!("P3");
        println!("{} {}", self.width, self.height);
        println!("255");
        for j in 0..self.height {
            for i in 0..self.width {
                print!("{}", self.colors[j * self.width + i].to_string())
            }
            println!()
        }
    }

}