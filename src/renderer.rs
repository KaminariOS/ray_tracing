use crate::camera::Camera;
use crate::types::{Color, SharedHittable};
use crate::Ray;
use cfg_if::cfg_if;
use derivative::Derivative;
use na::{Vector3, Vector4};
cfg_if! {
    if #[cfg(feature = "window")] {
use pixels::Pixels;
use crate::gui::Gui;
    }
}
use crate::rand_gen::get_rand;
#[cfg(feature = "window")]
use crate::scene::select_scene;

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
    background: Color
}

impl Renderer {
    pub fn new(width: u32, height: u32, (world, background): (SharedHittable, Color), camera: Camera) -> Self {
        Self {
            width,
            height,
            actual_width: width,
            scale: 1,
            camera,
            actual_height: height,
            world,
            multisample: 4,
            max_depth: 10,
            dirty: true,
            background
        }
    }

    pub fn draw(&mut self, frame: &mut [u8]) {
        if !self.dirty {
            return
        }
        self.dirty = false;
        log::info!("{:?}", self);
        let now = instant::Instant::now();
        let pixel_count = frame.len() / 4;

        assert_eq!(pixel_count as u32, self.width * self.height);

        let row_len = 4 * self.width as usize;
        cfg_if! {
            if #[cfg(feature = "rayon")] {
                use rayon::prelude::*;
                log::info!("Rayon enabled.");
                let iter = frame.par_chunks_exact_mut(row_len);
            } else {
                let iter = frame.chunks_exact_mut(row_len);
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
                    ProgressStyle::default_bar().template("{spinner:.green} [{elapsed_precise}] {wide_bar} {per_sec} {pos}/{len} rows eta: {eta}")
                );
                pb.set_draw_delta(if self.height > 500 {10} else {1});
                let iter = iter.progress_with(pb);
            }
        }

        iter.rev().enumerate().for_each(|(y, row)| {
            row.chunks_exact_mut(4).enumerate().for_each(|(x, pixel)|
                {
                    let rgba_float = (0..self.multisample)
                        .map(|_| {
                            let [u, v] = self.cal_norm_coords(x as u32, y as u32);
                            let ray = self.camera.get_ray(u, v);
                            self.ray_color(&ray, self.max_depth)
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
        self.camera.aspect_ratio = self.width as f32 / self.height as f32;
        self.camera.rebuild();
    }

    #[cfg(feature = "window")]
    pub(crate) fn update_scale(&mut self, scale: u32, pixels: &mut Pixels) {
        if scale != self.scale {
            self.scale = scale;
            self.resize(self.actual_width, self.actual_height, pixels);
        }
    }

    #[inline]
    fn cal_norm_coords(&self, x: u32, y: u32) -> [f32; 2] {
        let (x_offset, y_offset) = if self.multisample != 1 {
            (get_rand() - 0.5, get_rand() - 0.5)
        } else {
            (0., 0.)
        };
        [
            (x as f32 + x_offset) / (self.width - 1) as f32,
            (y as f32 + y_offset) / (self.height - 1) as f32,
        ]
    }

    #[cfg(feature = "window")]
    pub fn update_from_gui(&mut self, gui: &Gui, pixels: &mut Pixels) {
        self.update_scale(gui.scale, pixels);
        self.multisample = gui.sample_count;
        self.max_depth = gui.max_depth;
        let scene = gui.scene.to_str();
        if self.world.read().unwrap().get_label().filter(|&label| label == scene).is_none() {
            let (world, background) = select_scene(scene);
            self.world = world;
            self.background = background;
        }
    }
    fn ray_color(&self, r: &Ray, depth: usize) -> Color {
        if depth == 0 {
            return Color::zeros();
        }
        if let Some(hit_record) = self.world.read().unwrap().hit(r, 0.001, f32::INFINITY) {
            // let target = hit_record.normal + rand_vec3_on_unit_sphere();
            let emitted = hit_record.material.read().unwrap().emit(hit_record.uv, hit_record.point);
            let scattered = if let Some((attenuation, scattered)) = hit_record.material.read().unwrap().scatter(r, &hit_record) {
                attenuation.component_mul(&self.ray_color(&scattered, depth - 1))
            } else {
                Color::zeros()
            };
            emitted + scattered
        } else {
            self.background
        }

    }
}
