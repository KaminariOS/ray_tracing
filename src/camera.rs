use na::{Point3, Vector3};
use crate::Ray;

#[allow(dead_code)]
pub struct Camera {
    viewport_width: f32,
    viewport_height: f32,
    focal_length: f32,
    pub origin: Point3<f32>,
    pub horizontal: Vector3<f32>,
    pub vertical: Vector3<f32>,
    pub lower_left_corner: Point3<f32>,
}

impl Camera {
    pub fn new(viewport_height: f32, aspect_ratio: f32) -> Self {
        let viewport_width = viewport_height * aspect_ratio;
        let focal_length = 1.;
        let horizontal = Vector3::from([viewport_width, 0., 0.]);
        let vertical = Vector3::from([0., viewport_height, 0.]);
        let origin = Point3::origin();
        let lower_left_corner =
            origin - horizontal / 2. - vertical / 2. - Vector3::from([0., 0., focal_length]);
        Self {
            viewport_width,
            viewport_height,
            focal_length,
            origin,
            horizontal,
            vertical,
            lower_left_corner,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        let aspect_ratio = width as f32 / height as f32;
        *self = Camera::new(self.viewport_height, aspect_ratio);
    }

    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        Ray::new(
             self.origin,
             self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin
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
