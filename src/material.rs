use crate::rand_gen::{get_rand, rand_vec3_in_unit_sphere, rand_vec3_on_unit_sphere, random_cosine_direction};
use crate::ray::HitRecord;
use crate::texture::SolidColor;
use crate::types::{Color, create_shared_mut, RGB, Shared, SharedTexture};
use crate::Ray;
use na::{Point3, UnitVector3};
use std::f32::consts::PI;
use crate::onb::ONB;

pub trait Material: Sync + Send {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray, f32)>;
    fn emit(&self, _uv: [f32; 2], _point: Point3<f32>) -> Color {
        Color::zeros()
    }
    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f32 {
       1.
    }
}

pub struct Lambertian {
    albedo: SharedTexture,
}

impl Lambertian {
    pub fn new(albedo: SharedTexture) -> Shared<Self> {
        create_shared_mut(Lambertian { albedo })
    }
    pub fn from_color(color: RGB) -> Shared<Self> {
        Self::new(SolidColor::new(color))
    }
}

impl Material for Lambertian {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray, f32)> {
        let uvw = ONB::build_from_w(hit_record.normal);
        let mut scatter_dir = uvw.local_dir(random_cosine_direction());
        assert_eq!(uvw.w(), hit_record.normal);
        if near_zero(scatter_dir) {
            scatter_dir = hit_record.normal;
        }
        let scattered = Ray::new(hit_record.point, scatter_dir, ray_in.time);
        let pdf = uvw.w().dot(&scatter_dir) / PI;
        Some((
            self.albedo
                .read()
                .unwrap()
                .value(hit_record.uv, hit_record.point),
            scattered,
            pdf
        ))
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit_record.normal.dot(&scattered.direction);
        cosine.max(f32::EPSILON) / PI
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f32,
}

impl Metal {
    fn reflect(v: UnitVector3<f32>, n: UnitVector3<f32>) -> UnitVector3<f32> {
        UnitVector3::new_unchecked(v.into_inner() - 2. * n.dot(&v) * n.into_inner())
    }
    pub fn new(albedo: RGB, fuzz: f32) -> Shared<Self> {
        create_shared_mut(Metal {
            albedo: Color::from(albedo),
            fuzz: fuzz.min(1.),
        })
    }
}
impl Material for Metal {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray, f32)> {
        let reflected = Self::reflect(ray_in.direction, hit_record.normal);
        let scatter_dir = reflected.into_inner() + self.fuzz * rand_vec3_in_unit_sphere();
        let scattered =
            Ray::new(hit_record.point, UnitVector3::new_normalize(scatter_dir), ray_in.time);
        let pdf = self.scattering_pdf(ray_in, hit_record, &scattered);
        if hit_record.normal.dot(&scatter_dir) > 0. {
            Some((
                self.albedo,
                scattered,
                pdf
            ))
        } else {
            None
        }
    }
}

fn near_zero(vec: UnitVector3<f32>) -> bool {
    let eps = 1.0e-8;
    vec.iter().all(|x| x.abs() < eps)
}

pub struct Dielectric {
    pub(crate) index_of_refraction: f32,
}

impl Dielectric {
    pub fn new(index_of_refraction: f32) -> Shared<Self> {
        assert!(index_of_refraction > 0.);
        create_shared_mut(Self {
            index_of_refraction,
        })
    }
    fn refract(incident: UnitVector3<f32>, normal: UnitVector3<f32>, index_ratio: f32) -> UnitVector3<f32> {
        let cos_theta = (-incident.dot(&normal)).min(1.);
        let r_out_perp = index_ratio * (incident.into_inner() + cos_theta * normal.into_inner());
        let r_out_parallel = -(1. - r_out_perp.norm_squared()).abs().sqrt() * normal.into_inner();
        UnitVector3::new_normalize(r_out_perp + r_out_parallel)
    }

    fn reflectance(cosine: f32, ref_idx: f32) -> f32 {
        let r0 = (1. - ref_idx) / (1. + ref_idx);
        let r0 = r0 * r0;
        r0 + (1. - r0) * (1. - cosine).powf(5.)
    }
}

impl Material for Dielectric {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray, f32)> {
        let refraction_ratio = if hit_record.front_face {
            1. / self.index_of_refraction
        } else {
            self.index_of_refraction
        };
        let cos_theta = (-ray_in.direction.dot(&hit_record.normal)).min(1.);
        let sin_theta = (1. - cos_theta * cos_theta).sqrt();
        let cannot_refract = refraction_ratio * sin_theta > 1.;
        let direction =
            if cannot_refract || Self::reflectance(cos_theta, refraction_ratio) > get_rand() {
                Metal::reflect(ray_in.direction, hit_record.normal)
            } else {
                Self::refract(ray_in.direction, hit_record.normal, refraction_ratio)
            };
        let scattered =
            Ray::new(hit_record.point, direction, ray_in.time);
        let pdf = self.scattering_pdf(ray_in, hit_record, &scattered);
        Some((
            Color::repeat(1.),
            scattered,
            pdf
        ))
    }
}

pub struct DiffuseLight {
    texture: SharedTexture
}

impl DiffuseLight {
    pub fn new(texture: SharedTexture) -> Shared<Self> {
        create_shared_mut(Self{texture})
    }
    pub fn from_color(color: RGB) -> Shared<Self> {
        Self::new(SolidColor::new(color))
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<(Color, Ray, f32)> {
        None
    }
    fn emit(&self, uv: [f32; 2], point: Point3<f32>) -> Color {
        self.texture.read().unwrap().value(uv, point)
    }
}

pub struct Isotropic {
    albedo: SharedTexture
}

impl Isotropic {
    pub fn new(albedo: SharedTexture) -> Shared<Self> {
        create_shared_mut(Self{albedo})
    }

    #[allow(dead_code)]
    pub fn from_color(color: RGB) -> Shared<Self> {
        Self::new(SolidColor::new(color))
    }
}

impl Material for Isotropic {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<(Color, Ray, f32)> {
        let scattered = Ray::new(hit_record.point, rand_vec3_on_unit_sphere(), ray_in.time);
        let color = self.albedo.read().unwrap().value(hit_record.uv, hit_record.point);
        let pdf = self.scattering_pdf(ray_in, hit_record, &scattered);
        Some((color, scattered, pdf))
    }
}