use std::f32::consts::PI;
use crate::rand_gen::{get_rand, rand_vec3_in_unit_sphere, rand_vec3_on_unit_sphere};
use crate::ray::HitRecord;
use crate::texture::SolidColor;
use crate::types::{Color, create_shared_mut, RGB, Shared, SharedTexture};
use crate::Ray;
use na::UnitVector3;
use crate::pdf::{CosinePDF, PDF};

pub trait Material: Sync + Send {
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord>;
    fn emit(&self, _hit_record: &HitRecord) -> Option<Color> {
        None
    }
    fn scattering_pdf(&self, _ray_in: &Ray, _hit_record: &HitRecord, _scattered: &Ray) -> f32 {
       1.
    }
}
pub enum ScatterType {
    Specular(Ray),
    Diffuse(Box<dyn PDF>),
    ISO(Ray)
}
pub struct ScatterRecord {
    pub s_type: ScatterType,
    pub attenuation: Color,
}

impl ScatterRecord {
    pub fn new(s_type: ScatterType,attenuation: Color) -> Option<Self> {
        Some(Self {
            attenuation,
            s_type
        })
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
    fn scatter(&self, _ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let s_type = ScatterType::Diffuse(CosinePDF::new(hit_record.normal));
        ScatterRecord::new(
                s_type,
                self.albedo
                .read()
                .unwrap()
                .value(hit_record.uv,
                hit_record.point)
            )
    }

    fn scattering_pdf(&self, _ray_in: &Ray, hit_record: &HitRecord, scattered: &Ray) -> f32 {
        let cosine = hit_record.normal.dot(&scattered.direction);
        cosine.max(0.001) / PI
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
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let reflected = Self::reflect(ray_in.direction, hit_record.normal);
        let scatter_dir = reflected.into_inner() + self.fuzz * rand_vec3_in_unit_sphere();
        let scattered =
            Ray::new(hit_record.point, UnitVector3::new_normalize(scatter_dir), ray_in.time);
        let s_type = ScatterType::Specular(scattered);
        if hit_record.normal.dot(&scatter_dir) > 0. {
            ScatterRecord::new(
                s_type,
                self.albedo,
            )
        } else {
            None
        }
    }
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
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
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
        let s_type = ScatterType::Specular(scattered);
        ScatterRecord::new(
            s_type,
            Color::repeat(1.)
        )
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
    fn scatter(&self, _ray_in: &Ray, _hit_record: &HitRecord) -> Option<ScatterRecord> {
        None
    }
    fn emit(&self, hit_record: &HitRecord) -> Option<Color> {
        if hit_record.front_face {
            Some(self.texture.read().unwrap().value(hit_record.uv, hit_record.point))
        } else {None}
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
    fn scatter(&self, ray_in: &Ray, hit_record: &HitRecord) -> Option<ScatterRecord> {
        let scattered = Ray::new(hit_record.point, rand_vec3_on_unit_sphere(), ray_in.time);
        let color = self.albedo.read().unwrap().value(hit_record.uv, hit_record.point);
        let s_type = ScatterType::ISO(scattered);
        ScatterRecord::new(s_type, color)
    }
}