use crate::ray::{HitRecord, Hittable, HittableList};
use crate::{Color, Ray};
use na::Point3;
use std::sync::{Arc, RwLock};
use crate::material::{Dielectric, Lambertian, Material, Metal};

pub struct Sphere {
    pub center: Point3<f32>,
    pub radius: f32,
    material: Arc<dyn Material>
}

impl Sphere {
    pub fn new(center: Point3<f32>, radius: f32, material: Arc<dyn Material>) -> Arc<RwLock<Sphere>> {
        Arc::new(RwLock::new(Sphere { center, radius, material }))
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> bool {
        let oc = ray.origin - self.center;
        let a = ray.direction.norm_squared();
        let half_b = oc.dot(&ray.direction);
        let c = oc.norm_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0. {
            return false;
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return false;
            }
        }
        hit_record.t = root;
        hit_record.point = ray.at(root);
        let outward_normal = (hit_record.point - self.center) / self.radius;
        hit_record.set_face_normal(ray, outward_normal);
        hit_record.material = self.material.clone();
        true
    }
}

pub fn create_objs() -> HittableList {
    let material_ground = Lambertian::new(Color::from([0.8, 0.8, 0.]));
    let material_center = Lambertian::new(Color::from([0.1, 0.2, 0.5]));
    // let material_center = Dielectric::new(1.5);
    // let material_left = Metal::new(Color::from([0.8, 0.8, 0.8]), 0.3);
    let material_left = Dielectric::new(1.5);
    let material_right = Metal::new(Color::from([0.8, 0.6, 0.2]), 0.);
    HittableList {
        objects:
        vec![
            Sphere::new(Point3::from([0., 0., -1.]), 0.5, material_center),
            Sphere::new(Point3::from([0., -100.5, -1.]), 100., material_ground),
            Sphere::new(Point3::from([-1., 0., -1.]), 0.5, material_left.clone()),
            Sphere::new(Point3::from([-1., 0., -1.]), -0.4, material_left),
            Sphere::new(Point3::from([1., 0., -1.]), 0.5, material_right),
        ]
    }
}
