use na::{Point3, Vector3};
use crate::Ray;

#[allow(dead_code)]
pub struct Camera {
    pub origin: Point3<f32>,
    pub horizontal: Vector3<f32>,
    pub vertical: Vector3<f32>,
    pub lower_left_corner: Point3<f32>,
    vfov: f32,
    w: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    vup: Vector3<f32>,
    len_radius: f32,
    focus_dist: f32
}

use std::f32::consts::PI;
use crate::rand_gen::rand_vec3_in_unit_disk;

fn degree_to_radian(degree: f32) -> f32 {
    degree / 180.0 * PI
}
impl Camera {
    pub fn new(
        lookfrom: Point3<f32>,
        direction: Vector3<f32>,
        vup: Vector3<f32>,
        vfov: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_dist: f32
    ) -> Self {
        let theta = degree_to_radian(vfov);
        let h = (theta / 2.).tan();
        let viewport_height = 2. * h;
        let viewport_width = viewport_height * aspect_ratio;

        let w = -direction.normalize();
        let u = vup.cross(&w);
        let v = w.cross(&u);

        let horizontal = focus_dist * viewport_width * u;
        let vertical = focus_dist * viewport_height * v;
        let origin = lookfrom;
        let lower_left_corner =
            origin - horizontal / 2. - vertical / 2. - w * focus_dist;
        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner,
            vfov,
            vup,
            w, u, v,
            len_radius: aperture / 2.,
            focus_dist
        }
    }

    #[cfg(feature = "window")]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;
        *self = Camera::new(self.origin, -self.w, self.vup, self.vfov, aspect_ratio, self.len_radius * 2., self.focus_dist);
    }

    pub fn get_ray(&self, s: f32, t: f32) -> Ray {
        let rd = self.len_radius * rand_vec3_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(
             self.origin + offset,
             self.lower_left_corner + s * self.horizontal + t * self.vertical - self.origin - offset
        )
    }

    // fn get_horizontal(&self) -> Vector3<f32> {
    //     Vector3::from([self.viewport_width, 0., 0.])
    // }
    //
    // fn get_vertical(&self) -> Vector3<f32> {
    //     Vector3::from([0., self.viewport_height, 0.])
    // }
    //
    // fn get_lower_left_corner(&self) -> Point3<f32> {
    //     self.origin - self.get_horizontal() / 2. - self.get_vertical() / 2. - Vector3::from([0., 0., self.focal_length])
    // }
}
