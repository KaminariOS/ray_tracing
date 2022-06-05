use crate::camera::Camera;
use crate::ray::{HitRecord, Hittable, HittableList};
use crate::{Color, Ray};
use cfg_if::cfg_if;
use na::{Vector3, Vector4};
use pixels::Pixels;
use crate::gui::Gui;
use crate::rand_gen::{get_rand};

pub struct Renderer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    camera: Camera,
    scale: u32,
    actual_width: u32,
    actual_height: u32,
    world: HittableList,
    pub(crate) multisample: usize,
    pub(crate) max_depth: usize
}

impl Renderer {
    pub fn new(width: u32, height: u32, world: HittableList) -> Renderer {
        let aspect_ratio = width as f32 / height as f32;
        let viewport_height = 2.;
        Self {
            width,
            height,
            actual_width: width,
            scale: 1,
            camera: Camera::new(viewport_height, aspect_ratio),
            actual_height: height,
            world,
            multisample: 4,
            max_depth: 10
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {

        let pixel_count = frame.len() / 4;
        cfg_if! {
            if #[cfg(feature = "progress")] {
                use indicatif::ProgressBar;
                let work_div = pixel_count / 100;
                let pb = ProgressBar::new( (pixel_count / work_div) as u64);
            }
        }
        assert_eq!(pixel_count as u32, self.width * self.height);
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let iter = frame.chunks_exact_mut(4);
            } else {
                use rayon::prelude::*;
                let iter = frame.par_chunks_exact_mut(4);
            }
        }

        iter.enumerate().for_each(|(i, pixel)| {
            #[cfg(feature = "progress")]
            if i % work_div == 0 {
                pb.inc(1);
            }
            let (x, y) = self.cal_coords(i);
            // let rgba_float = self.draw_gradient(u, v);
            // let rgba_float = self.draw_checkerboard(u, v);
            let rgba_float = (0..self.multisample)
                .map(|_| {
                    let (u, v) = self.cal_norm_coords(x, y);
                    let ray = self.camera.get_ray(u, v);
                    ray_color(&ray, &self.world, self.max_depth)
                })
                .fold(Vector4::zeros(), |acc, next| acc + next) / self.multisample as f32;
            let mut rgba = rgba_float
                .into_iter()
                .map(Self::float_to_rgb)
                .collect::<Vec<_>>();
            rgba[3] = 0xff;
            pixel.copy_from_slice(&rgba);
        });

        #[cfg(feature = "progress")]
        pb.finish_with_message("Done");
    }

    #[allow(dead_code)]
    fn draw_checkerboard(&self, u: f32, v: f32) -> Vector4<f32> {
        let (x, y) = self.norm_to_integer(u, v);
        let scale = 4;
        let color = ((x >> scale) % 2) ^ ((y >> scale) % 2);
        let color = color as f32;
        Vector4::from([color, color, color, 1.])
    }

    #[inline]
    fn float_to_rgb(num: &f32) -> u8 {
        (num.max(0.).min(0.999).sqrt() * 256.) as u8
    }

    #[inline]
    fn norm_to_integer(&self, u: f32, v: f32) -> (u32, u32) {
        let x = (u * (self.width as f32 - 1.)) as u32;
        let y = (v * (self.height as f32 - 1.)) as u32;
        (x, y)
    }

    #[allow(dead_code)]
    fn draw_gradient(&self, u: f32, v: f32) -> Vector4<f32> {
        let (r, g) = (u, v);
        let b = 0.23;

        Vector4::from([r, g, b, 1.])
    }
    pub(crate) fn resize(&mut self, width: u32, height: u32, pixels: &mut Pixels) {
        pixels.resize_surface(width, height);
        self.actual_height = height;
        self.actual_width = width;
        self.width = width / self.scale;
        self.height = height / self.scale;
        pixels.resize_buffer(self.width, self.height);
        self.camera.resize(self.width, self.height);
    }

    pub(crate) fn update_scale(&mut self, scale: u32, pixels: &mut Pixels) {
        if scale != self.scale {
            self.scale = scale;
            self.resize(self.actual_width, self.actual_height, pixels);
        }
    }

    #[inline]
    fn cal_coords(&self, i: usize) -> (u32, u32) {
        let i = i as u32;
        (i % self.width, self.height - 1 - i / self.width)
    }

    #[inline]
    fn cal_norm_coords(&self, x: u32, y: u32) -> (f32, f32) {
        let (x_offset, y_offset) = if self.multisample != 1 {
            (get_rand() - 0.5, get_rand() - 0.5)
        } else {
            (0., 0.)
        };
        (
            (x as f32 + x_offset) / (self.width - 1) as f32,
            (y as f32 + y_offset) / (self.height - 1) as f32,
        )
    }

    pub fn update_from_gui(&mut self, gui: &Gui, pixels: &mut Pixels) {
        self.update_scale(gui.scale, pixels);
        self.multisample = gui.sample_count;
        self.max_depth = gui.max_depth;
    }
}

fn ray_color(r: &Ray, world: &HittableList, depth: usize) -> Vector4<f32> {
    if depth == 0 {
        return Vector3::zeros().push(1.)
    }
    let mut hit_record = HitRecord::default();
    if world.hit(r, 0.001, f32::INFINITY, &mut hit_record) {
        // let target = hit_record.normal + rand_vec3_on_unit_sphere();
        let mut scattered = Ray::default();
        let mut attenuation = Color::zeros();
        if hit_record.material.scatter(r, &hit_record, &mut attenuation, &mut scattered) {
            return attenuation.push(1.).component_mul(&ray_color(&scattered, world, depth - 1))
        }
        return Vector4::zeros()
    }
    let unit_direction = r.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.);
    let color = (1. - t) * Vector3::from([1.; 3]) + t * Vector3::from([0.5, 0.7, 1.]);
    color.push(1.)
}