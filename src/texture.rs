use na::Point3;
use crate::types::{Color, create_shared_mut, Shared, SharedTexture};

pub trait Texture: Sync + Send {
    fn value(&self, uv: (f32, f32), p: Point3<f32>) -> Color;
}

pub struct SolidColor {
    color: Color
}

impl SolidColor {
    pub fn new(color: Color) -> Shared<Self> {
        create_shared_mut(Self{color})
    }
}

impl Texture for SolidColor {
    fn value(&self, _uv: (f32, f32), _p: Point3<f32>) -> Color {
        self.color
    }
}

pub struct CheckerTexture {
    odd: SharedTexture,
    even: SharedTexture
}

impl CheckerTexture {
    pub fn new(even: Color, odd: Color) -> Shared<Self> {
        create_shared_mut(Self {
            odd: SolidColor::new(odd),
            even: SolidColor::new(even)
        })
    }
}

impl Texture for CheckerTexture {
    fn value(&self, uv: (f32, f32), p: Point3<f32>) -> Color {
        let sines = p.iter().fold(1.0, |acc, &cur| acc * (10. * cur).sin());
        let tex = if sines < 0. {&self.odd} else {&self.even};
        tex.read().unwrap().value(uv , p)
    }
}