use crate::ray::{HitRecord, Hittable, HittableList};
use crate::{Color, Ray};
use na::{Point3, Vector3};
use std::sync::{Arc, RwLock};
use crate::material::{Dielectric, Lambertian, Material, Metal};
use crate::rand_gen::{get_rand, get_rand_range, get_rand_vec3_range};

pub struct Sphere {
    pub center0: Point3<f32>,
    pub center1: Point3<f32>,
    pub radius: f32,
    material: Arc<dyn Material>,
    time0: f32,
    time1: f32,
    moving: bool
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32, material: Arc<dyn Material>) -> Arc<RwLock<Sphere>> {
        Arc::new(RwLock::new(Sphere {
            center0: Point3::from(center),
            center1: Point3::from(center),
            time0: 0.,
            time1: 0.,
            radius,
            material,
            moving: false
        }))
    }
    pub fn new_moving(center0: [f32; 3], center1: [f32; 3], time0: f32, time1: f32, radius: f32, material: Arc<dyn Material>) -> Arc<RwLock<Sphere>>{
        float_eq::assert_float_ne!(time0, time1, rmin <= f32::EPSILON);
        Arc::new(RwLock::new(Sphere {
            center0: Point3::from(center0),
            center1: Point3::from(center1),
            time0,
            time1,
            radius,
            material,
            moving: true
        }))
    }
    fn get_center(&self, time: f32) -> Point3<f32> {
         if self.moving {
            self.center0 + (time - self.time0) / (self.time1 - self.time0) * (self.center1 - self.center0)
        }
        else {
            self.center0
        }
}
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32, hit_record: &mut HitRecord) -> bool {
        let oc = ray.origin - self.get_center(ray.time);
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
        let outward_normal = (hit_record.point - self.get_center(ray
            .time)) / self.radius;
        hit_record.set_face_normal(ray, outward_normal);
        hit_record.material = self.material.clone();
        true
    }
}

#[allow(dead_code)]
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
            Sphere::new([0., 0., -1.], 0.5, material_center),
            Sphere::new([0., -100.5, -1.], 100., material_ground),
            Sphere::new([-1., 0., -1.], 0.5, material_left.clone()),
            Sphere::new([-1., 0., -1.], -0.45, material_left),
            Sphere::new([1., 0., -1.], 0.5, material_right),
        ]
    }
}

pub fn create_random_scene() -> HittableList {
    let mut objects: Vec<_> = (-11..11)
        .map(|a|
            (-11..11)
                .filter_map(move |b|
                             create_random_sphere(a, b)
                )
        ).flatten()
        .map(|x| x as Arc<RwLock<dyn Hittable>>)
        .collect();

    let material_ground = Lambertian::new(Color::from([0.5, 0.5, 0.5]));
    let material1 = Dielectric::new(1.5);
    let material2 = Lambertian::new(Color::from([0.4, 0.2, 0.1]));
    let material3 = Metal::new(Color::from([0.7, 0.6, 0.5]), 0.);
    let vec: Vec<Arc<RwLock<dyn Hittable>>> = vec![
        Sphere::new([0., -1000., 0.], 1000., material_ground),
        Sphere::new([0., 1., 0.], 1., material1),
        Sphere::new([-4., 1., 0.], 1., material2),
        Sphere::new([4., 1., 0.], 1., material3)
    ];
    objects.extend(vec);

    HittableList {
        objects
    }
}

fn create_random_sphere(a: i32, b: i32) -> Option<Arc<RwLock<Sphere>>>{
    let a = a as f32;
    let b = b as f32;
    let mat = get_rand();
    let center = Point3::from([a + 0.9 * get_rand(), 0.2, b + 0.9 * get_rand()]);
    let mut moving = false;
    if (center - Point3::from([4., 0.2, 0.])).norm() > 0.9 {
        let material: Arc<dyn Material> = if mat < 0.8 {
            let albedo = get_rand_vec3_range(0., 1.).component_mul(&get_rand_vec3_range(0., 1.));
            moving = true;
            Lambertian::new(albedo)
        } else if mat < 0.95 {
            let albedo = get_rand_vec3_range(0.5, 1.);
            let fuzz = get_rand_range(0., 0.5);
            Metal::new(albedo, fuzz)
        } else {
            Dielectric::new(1.5)
        };
        Some(if moving {
            let center2 = center + Vector3::from([0., get_rand_range(0., 0.5), 0.]);
              Sphere::new_moving(center.into(), center2.into(), 0., 1., 0.2, material)
        } else {
            Sphere::new(center.into(), 0.2, material)
        })
    }else { None }
}