use crate::aabb::AxisAlignedBoundingBox;
use crate::ray::{HitRecord, Hittable};
use crate::types::{create_shared_mut, SharedMaterial, SharedSphere};
use crate::Ray;
use na::{Point3, Vector3};

const PI: f32 = std::f32::consts::PI;

pub struct Sphere {
    pub center0: Point3<f32>,
    pub center1: Point3<f32>,
    pub radius: f32,
    material: SharedMaterial,
    time0: f32,
    time1: f32,
    moving: bool,
    label: Option<String>
}

impl Sphere {
    pub fn new(center: [f32; 3], radius: f32, material: SharedMaterial) -> SharedSphere {
        Self::new_with_label(center, radius, material, None)
    }
    pub fn new_with_label(center: [f32; 3], radius: f32, material: SharedMaterial, label: Option<String>) -> SharedSphere {

        create_shared_mut(Sphere {
            center0: Point3::from(center),
            center1: Point3::from(center),
            time0: 0.,
            time1: 0.,
            radius,
            material,
            moving: false,
            label
        })
    }
    pub fn new_moving(
        center0: [f32; 3],
        center1: [f32; 3],
        time0: f32,
        time1: f32,
        radius: f32,
        material: SharedMaterial,
    ) -> SharedSphere {
        float_eq::assert_float_ne!(time0, time1, rmin <= f32::EPSILON);
        create_shared_mut(Sphere {
            center0: Point3::from(center0),
            center1: Point3::from(center1),
            time0,
            time1,
            radius,
            material,
            moving: true,
            label: None
        })
    }
    fn get_center(&self, time: f32) -> Point3<f32> {
        if self.moving {
            self.center0
                + (time - self.time0) / (self.time1 - self.time0) * (self.center1 - self.center0)
        } else {
            self.center0
        }
    }

    fn get_sphere_uv(p: Point3<f32>) -> [f32; 2] {
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + PI;
        [phi / (2. * PI), theta / PI]
    }
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let oc = ray.origin - self.get_center(ray.time);
        let a = ray.direction.norm_squared();
        let half_b = oc.dot(&ray.direction);
        let c = oc.norm_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0. {
            return None;
        }
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }
        let mut hit_record = HitRecord::default();
        hit_record.t = root;
        hit_record.point = ray.at(root);
        let outward_normal = (hit_record.point - self.get_center(ray.time)) / self.radius;
        hit_record.uv = Self::get_sphere_uv(outward_normal.into());
        hit_record.set_face_normal(ray, outward_normal);
        hit_record.material = self.material.clone();
        Some(hit_record)
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox> {
        let offset = Vector3::repeat(self.radius);
        if self.moving {
            let center0 = self.get_center(time0);
            let center1 = self.get_center(time1);
            let bbox0 = AxisAlignedBoundingBox::new(center0 - offset, center0 + offset);
            let bbox1 = AxisAlignedBoundingBox::new(center1 - offset, center1 + offset);
            AxisAlignedBoundingBox::surrounding_box(Some(bbox0), Some(bbox1))
        } else {
            Some(AxisAlignedBoundingBox::new(
                self.center0 - offset,
                self.center0 + offset,
            ))
        }
    }

    fn get_label(&self) -> Option<&String> {
        self.label.as_ref()
    }
}
