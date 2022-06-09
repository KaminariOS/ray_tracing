use image::{Pixel, RgbaImage};
use crate::types::{create_shared_mut, Color, Shared, SharedTexture};
use na::Point3;
use crate::perlin::Perlin;
use crate::resource;

pub trait Texture: Sync + Send {
    fn value(&self, uv: [f32; 2], p: Point3<f32>) -> Color;
}

pub struct SolidColor {
    color: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Shared<Self> {
        create_shared_mut(Self { color })
    }
}

impl Texture for SolidColor {
    fn value(&self, _uv: [f32; 2], _p: Point3<f32>) -> Color {
        self.color
    }
}

pub struct CheckerTexture {
    odd: SharedTexture,
    even: SharedTexture,
}

impl CheckerTexture {
    pub fn new(even: Color, odd: Color) -> Shared<Self> {
        create_shared_mut(Self {
            odd: SolidColor::new(odd),
            even: SolidColor::new(even),
        })
    }
}

impl Texture for CheckerTexture {
    fn value(&self, uv: [f32; 2], p: Point3<f32>) -> Color {
        let sines = p.iter().fold(1.0, |acc, &cur| acc * (10. * cur).sin());
        let tex = if sines < 0. { &self.odd } else { &self.even };
        tex.read().unwrap().value(uv, p)
    }
}

pub struct NoiseTexture {
    noise: Perlin,
    scale: f32
}

impl NoiseTexture {
    pub fn new(scale: f32) -> Shared<Self> {
        create_shared_mut(Self{
            noise: Perlin::new(),
            scale
        })
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _uv: [f32; 2], p: Point3<f32>) -> Color {
       // Color::from([1., 1., 1.]) * self.noise.turb(self.scale * p, None)

        Color::from([1., 1., 1.]) * 0.5 * (1. + (self.scale * p.z + 10. * self.noise.turb(p, None)).sin())
    }
}

pub struct ImageTexture {
    img: Option<RgbaImage>
}

impl ImageTexture {
    const COLOR_SCALE: f32 = 1. / 255.;
    pub fn new(filename: &str) -> Shared<Self> {
        let img = resource::load_binary(filename).and_then(|bytes| Self::from_bytes(&bytes)).ok();
        create_shared_mut(Self {img})
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<RgbaImage> {
        let mut img = image::load_from_memory(bytes)?.to_rgba8();
        image::imageops::flip_vertical_in_place(&mut img);
        Ok(img)
    }

}

impl Texture for ImageTexture {
    fn value(&self, uv: [f32; 2], _p: Point3<f32>) -> Color {
        if let Some(img) = &self.img {
            let (w, h) = img.dimensions();
            let xy: Vec<_>= uv.into_iter().zip([w, h])
                .map(|(x, d)| ((x.clamp(0., 1.) * d as f32) as u32).min(d - 1)
            ).collect();
            let p = img.get_pixel(xy[0], xy[1]).to_rgb().0.map(|x| x as f32 * Self::COLOR_SCALE);
            Color::from(p)
        } else {
            Color::from([0., 1., 1.])
        }
    }
}
