#[derive(Clone, Copy)]
pub struct Color{
    pub r: u8,
    pub g: u8,
    pub b: u8
}

impl Color {
    pub fn to_string(&self) -> String {
        format!("{}\t{}\t{}\t", self.r, self.g, self.b)
    }
}
