use crate::camera::Camera;
use crate::types::{Color, SharedHittable};
use crate::Ray;
use cfg_if::cfg_if;
use derivative::Derivative;
use na::{Point3, Vector3, Vector4};
cfg_if! {
    if #[cfg(feature = "window")] {
use pixels::Pixels;
use crate::gui::Gui;
    }
}
use crate::rand_gen::get_rand;

#[allow(dead_code)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Renderer {
    pub(crate) width: u32,
    pub(crate) height: u32,
    #[derivative(Debug = "ignore")]
    camera: Camera,
    scale: u32,
    actual_width: u32,
    actual_height: u32,
    #[derivative(Debug = "ignore")]
    world: SharedHittable,
    pub(crate) multisample: usize,
    pub(crate) max_depth: usize,
    #[derivative(Debug = "ignore")]
    pub dirty: bool,
}

impl Renderer {
    pub fn new(width: u32, height: u32, world: SharedHittable) -> Renderer {
        let aspect_ratio = width as f32 / height as f32;
        let aperture = 0.1;
        let dist_to_focus = 10.;
        Self {
            width,
            height,
            actual_width: width,
            scale: 1,
            camera: Camera::new(
                Point3::from([13., 2., 3.]),
                Vector3::from([-13., -2., -3.]),
                Vector3::y(),
                20.,
                aspect_ratio,
                aperture,
                dist_to_focus,
                0.,
                1.,
            ),
            actual_height: height,
            world,
            multisample: 4,
            max_depth: 10,
            dirty: true,
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        log::info!("{:?}", self);
        let now = instant::Instant::now();
        let pixel_count = frame.len() / 4;

        assert_eq!(pixel_count as u32, self.width * self.height);

        let line_len = 4 * self.width as usize;
        cfg_if! {
            if #[cfg(feature = "rayon")] {
                use rayon::prelude::*;
                let iter = frame.par_chunks_exact_mut(line_len);
            } else {
                let iter = frame.chunks_exact_mut(line_len);
            }
        }

        cfg_if! {
            if #[cfg(feature = "progress")] {
                use indicatif::{ProgressBar, ProgressStyle};
                cfg_if! {
                    if #[cfg(feature = "rayon")] {
                        use indicatif::ParallelProgressIterator;
                    } else {
                        use indicatif::ProgressIterator;
                    }
                }
                let pb = ProgressBar::new(self.height as u64);
                pb.set_style(
                    ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] {wide_bar} {pos}/{len} rows")
                );
                let iter = iter.progress_with(pb);
                // pb.set_draw_delta(10 as u64);
            }
        }

        iter.rev().enumerate().for_each(|(y, line)| {
            line.chunks_exact_mut(4).enumerate().for_each(|(x, pixel)|
                {
                    let rgba_float = (0..self.multisample)
                        .map(|_| {
                            let (u, v) = self.cal_norm_coords(x as u32, y as u32);
                            let ray = self.camera.get_ray(u, v);
                            ray_color(&ray, &self.world, self.max_depth)
                        })
                        .fold(Vector3::zeros(), |acc, next| acc + next)
                        / self.multisample as f32;
                    let mut rgb = rgba_float
                        .into_iter()
                        .map(Self::float_to_rgb)
                        .collect::<Vec<_>>();
                    rgb.push(0xff);
                    pixel.copy_from_slice(&rgb);
                }
            )
        });

        let seconds = now.elapsed().as_secs();
        log::info!("Time: {}min {}s", seconds / 60, seconds % 60);
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
    #[cfg(feature = "window")]
    pub(crate) fn resize(&mut self, width: u32, height: u32, pixels: &mut Pixels) {
        pixels.resize_surface(width, height);
        self.actual_height = height;
        self.actual_width = width;
        self.width = width / self.scale;
        self.height = height / self.scale;
        pixels.resize_buffer(self.width, self.height);
        self.camera.resize(self.width, self.height);
    }

    #[cfg(feature = "window")]
    pub(crate) fn update_scale(&mut self, scale: u32, pixels: &mut Pixels) {
        if scale != self.scale {
            self.scale = scale;
            self.resize(self.actual_width, self.actual_height, pixels);
        }
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

    #[cfg(feature = "window")]
    pub fn update_from_gui(&mut self, gui: &Gui, pixels: &mut Pixels) {
        self.update_scale(gui.scale, pixels);
        self.multisample = gui.sample_count;
        self.max_depth = gui.max_depth;
    }
}

fn ray_color(r: &Ray, world: &SharedHittable, depth: usize) -> Vector3<f32> {
    if depth == 0 {
        return Color::zeros();
    }
    if let Some(hit_record) = world.read().unwrap().hit(r, 0.001, f32::INFINITY) {
        // let target = hit_record.normal + rand_vec3_on_unit_sphere();
        if let Some((attenuation, scattered)) = hit_record.material.scatter(r, &hit_record) {
            return attenuation.component_mul(&ray_color(&scattered, world, depth - 1));
        }
        return Color::zeros();
    }
    let unit_direction = r.direction.normalize();
    let t = 0.5 * (unit_direction.y + 1.);
    let color = (1. - t) * Color::from([1.; 3]) + t * Color::from([0.5, 0.7, 1.]);
    color
}
