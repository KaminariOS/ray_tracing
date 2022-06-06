use std::sync::Arc;
use na::Vector3;
use crate::{Color, Ray};
use crate::rand_gen::{get_rand, rand_vec3_in_unit_sphere, rand_vec3_on_unit_sphere};
use crate::ray::HitRecord;

pub trait Material: Sync + Send {
   fn scatter(&self, ray: &Ray, hit_record: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool;
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
   albedo: Color,
   fuzz: f32
}

impl Metal {
   fn reflect(v: Vector3<f32>, n: Vector3<f32>) -> Vector3<f32> {
      v - 2. * n.dot(&v) * n
   }
   pub fn new(albedo: Color, fuzz: f32) -> Arc<Self> {
      Arc::new(Metal {albedo, fuzz: fuzz.min(1.)} )
   }
}
impl Material for Metal {
   fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord, attenuation: &mut Vector3<f32>, scattered: &mut Ray) -> bool {
      let reflected = Self::reflect(ray_in.direction, hit_record.normal);
      *scattered = Ray::new(hit_record.point, reflected + self.fuzz * rand_vec3_in_unit_sphere());
      *attenuation = self.albedo;
      hit_record.normal.dot(&scattered.direction) > 0.
   }
}

fn near_zero(vec: Vector3<f32>) -> bool {
   let eps = 1.0e-8;
   vec.iter().all(|x| x.abs() < eps)
}

pub struct Dielectric {
   pub(crate) index_of_refraction: f32
}

impl Dielectric {

   pub fn new(index_of_refraction: f32) -> Arc<Self> {
      assert!(index_of_refraction > 0.);
      Arc::new(Self {
         index_of_refraction
      })
   }
   fn refract(incident: Vector3<f32>, normal: Vector3<f32>, index_ratio: f32) -> Vector3<f32> {
      let cos_theta = (-incident.dot(&normal)).min(1.);
      let r_out_perp = index_ratio * (incident + cos_theta * normal);
      let r_out_parallel = -(1. - r_out_perp.norm_squared()).abs().sqrt() * normal;
      r_out_perp + r_out_parallel
   }

   fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
      let r0 = (1. - ref_idx) / (1. + ref_idx);
      let r0 = r0 * r0;
      r0 + (1. - r0) * (1. - cosine).powf(5.)
   }
}

impl Material for Dielectric {
   fn scatter(&self, ray: &Ray, hit_record: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
      *attenuation = Vector3::repeat(1.);
      let refraction_ratio = if hit_record.front_face {1. / self.index_of_refraction} else { self.index_of_refraction };
      let cos_theta = (-ray.direction.dot(&hit_record.normal)).min(1.);
      let sin_theta = (1. - cos_theta * cos_theta).sqrt();
      let cannot_refract = refraction_ratio * sin_theta > 1.;
      let direction = if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > get_rand()
      {
         Metal::reflect(ray.direction, hit_record.normal)
      } else {
         Self::refract(ray.direction, hit_record.normal, refraction_ratio)
      };
      *scattered = Ray::new(hit_record.point, direction);
      true
   }
}
