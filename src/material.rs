use std::sync::Arc;
use na::Vector3;
use crate::{Color, Ray};
use crate::rand_gen::{rand_vec3_on_unit_sphere};
use crate::ray::HitRecord;

pub trait Material: Sync + Send {
   fn scatter(&self, ray: &Ray, hit_record: &HitRecord, attenuation: &mut Vector3<f32>, scattered: &mut Ray) -> bool;
}

pub struct Lambertian {
   albedo: Color
}

impl Lambertian {
   pub fn new(albedo: Color) -> Arc<Self> {
    Arc::new(Lambertian {albedo} )
   }
}
impl Material for Lambertian {
   fn scatter(&self, _ray: &Ray, hit_record: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
      let mut scatter_dir = hit_record.normal + rand_vec3_on_unit_sphere();
      if near_zero(scatter_dir) {
         scatter_dir = hit_record.normal;
      }
       *scattered = Ray::new(hit_record.point, scatter_dir);
      *attenuation = self.albedo;
      true
   }
}

pub struct Metal {
   albedo: Color
}

impl Metal {
   fn reflect(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
      v - 2. * n.dot(&v) * n
   }
   pub fn new(albedo: Color) -> Arc<Self> {
      Arc::new(Metal {albedo} )
   }
}
impl Material for Metal {
   fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord, attenuation: &mut Vector3<f32>, scattered: &mut Ray) -> bool {
      let reflected = Self::reflect(ray_in.direction, hit_record.normal);
      *scattered = Ray::new(hit_record.point, reflected);
      *attenuation = self.albedo;
      hit_record.normal.dot(&scattered.direction) > 0.
   }
}

fn near_zero(vec: Vector3<f32>) -> bool {
   let eps = 1.0e-8;
   vec.iter().all(|x| x.abs() < eps)
}
