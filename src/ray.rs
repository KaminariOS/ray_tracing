use na::{Point3, Vector3};
use std::sync::{Arc, RwLock};
use crate::Color;
use crate::material::{Lambertian, Material};

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32
}

impl Ray {
    pub fn at(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
    }
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, time: f32) -> Self {
        assert_ne!(direction.norm_squared(), 0.);
        Self {
            origin,
            direction: direction.normalize(),
            time
        }
    }
}
impl Default for Ray {
    fn default() -> Self {
        Self::new(Point3::origin(), Vector3::identity(), 0.)
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
}

pub struct HitRecord {
    pub(crate) point: Point3<f32>,
    pub(crate) normal: Vector3<f32>,
    pub(crate) t: f32,
    pub front_face: bool,
    pub material: Arc<dyn Material>
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            point: Point3::origin(),
            normal: Vector3::zeros(),
            t: f32::MAX,
            front_face: false,
            material: Lambertian::new(Color::from([0.8, 0.8, 0.]))
        }
    }
}

impl HitRecord {
    pub fn set_face_normal(&mut self, ray: &Ray, outward_normal: Vector3<f32>) {
        let outward_normal = outward_normal.normalize();
        self.front_face = ray.direction.dot(&outward_normal) < 0.;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub struct HittableList {
    pub objects: Vec<Arc<RwLock<dyn Hittable>>>,
}

impl HittableList {
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.objects.clear()
    }
    #[allow(dead_code)]
    pub fn add(&mut self, object: Arc<RwLock<dyn Hittable>>) {
        self.objects.push(object)
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self { objects: vec![] }
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_temp = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        for object in &self.objects {
            if let Some(new_hit) = object
                .read()
                .unwrap()
                .hit(ray, t_min, closest_so_far)
            {
                hit_temp = new_hit;
                hit_anything = true;
                closest_so_far = hit_temp.t;
            }
        }
        if hit_anything {
            Some(hit_temp)
        } else {
            None
        }
    }
}
