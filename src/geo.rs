use crate::aabb::AxisAlignedBoundingBox;
use crate::ray::{HitRecord, Hittable, HittableList};
use crate::types::{create_shared_mut, RGB, Shared, SharedHittable, SharedMaterial, SharedSphere, SharedTexture};
use crate::{camera, Ray};
use na::{Point3, Rotation3, UnitVector3, Vector3};
use strum::{EnumIter, IntoEnumIterator};
use crate::material::Isotropic;
use crate::rand_gen::get_rand;
use crate::texture::SolidColor;

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
        hit_record.set_face_normal(ray, UnitVector3::new_normalize(outward_normal));
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

#[derive(EnumIter, Clone, Copy, Debug)]
pub enum AlignedAxis {
    XY,
    XZ,
    YZ,
}

impl AlignedAxis {
    fn get_indexes(&self) -> [usize; 3] {
        match self {
            Self::XY => [0, 1, 2],
            Self::XZ => [0, 2, 1],
            Self::YZ => [1, 2, 0]
        }
    }
}
pub struct AxisAlignedRect {
    material: SharedMaterial,
    k: f32,
    p0: Point3<f32>,
    p1: Point3<f32>,
    axis: AlignedAxis
}


impl AxisAlignedRect {
    pub fn new(material: SharedMaterial, k: f32, c0: [f32; 2], c1: [f32; 2], axis: AlignedAxis) -> Shared<Self> {
        let axis_map = axis.get_indexes();
        let mut p0 = Point3::origin();
        let mut p1 = Point3::origin();
        assert!(c0[0] < c1[0] && c0[1] < c1[1]);
        for i in 0..2 {
            p0[axis_map[i]] = c0[i];
            p1[axis_map[i]] = c1[i];
        }
        p0[axis_map[2]] = k - 0.0001;
        p1[axis_map[2]] = k + 0.0001;
        create_shared_mut(Self {
            material, k, p0, p1, axis
        })
    }
}

impl Hittable for AxisAlignedRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let [xi, yi, zi] = self.axis.get_indexes();
        let t = (self.k - ray.origin[zi]) / ray.direction[zi];
        if t < t_min || t > t_max {return None}
        let xyz = ray.at(t);
        let [x, y] = [xyz[xi], xyz[yi]];
        if x < self.p0[xi] || x > self.p1[xi] || y < self.p0[yi] || y > self.p1[yi] {
            return None
        }
        let mut hit_record = HitRecord::default();
        let upper = xyz - self.p0;
        let lower = self.p1 - self.p0;
        hit_record.uv = [upper[xi] / lower[xi], upper[yi] / lower[yi]];
        let mut outward_normal = Vector3::zeros();
        outward_normal[zi] = 1.;
        hit_record.set_face_normal(ray, UnitVector3::new_unchecked(outward_normal));
        hit_record.material = self.material.clone();
        hit_record.point = xyz;
        hit_record.t = t;
        Some(hit_record)
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<AxisAlignedBoundingBox> {
        let output_box = AxisAlignedBoundingBox::new(self.p0, self.p1);
        Some(output_box)
    }
}

pub struct Cuboid {
    cuboid_min: Point3<f32>,
    cuboid_max: Point3<f32>,
    sides: SharedHittable,
}

impl Cuboid {
    pub fn new(cuboid_min: [f32; 3], cuboid_max: [f32; 3], material: SharedMaterial) -> Shared<Self> {
        let sides = AlignedAxis::iter()
            .map(|axis| {
                let axis_map = axis.get_indexes();
                let c0 = [cuboid_min[axis_map[0]], cuboid_min[axis_map[1]]];
                let c1 = [cuboid_max[axis_map[0]], cuboid_max[axis_map[1]]];
                let ks = [cuboid_min[axis_map[2]], cuboid_max[axis_map[2]]];
                ks.into_iter().map(|k|
                        AxisAlignedRect::new(material.clone(),
                                             k,
                                             c0,
                                             c1,
                                             axis
                        ) as SharedHittable
                ).collect::<Vec<_>>()
            }
        ).flatten().collect::<Vec<_>>();
        let sides = HittableList::new(sides, None);
    create_shared_mut(Self {
        cuboid_min: Point3::from(cuboid_min),
        cuboid_max: Point3::from(cuboid_max),
        sides
    })
    }
}

impl Hittable for Cuboid {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.sides.read().unwrap().hit(ray, t_min, t_max)
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<AxisAlignedBoundingBox> {
        Some(AxisAlignedBoundingBox::new(self.cuboid_min, self.cuboid_max))
    }
}

pub struct Translation {
    obj: SharedHittable,
    offset: Vector3<f32>
}

impl Translation {
    pub fn new(obj: SharedHittable, offset: [f32; 3]) -> Shared<Self> {
        create_shared_mut(Self {
            obj,
            offset: Vector3::from(offset)
        })
    }
}

impl Hittable for Translation {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);
        if let Some(mut hit_record) = self.obj.read().unwrap().hit(&moved_ray, t_min, t_max) {
            hit_record.point += self.offset;
            // hit_record.set_face_normal(&moved_ray, hit_record.normal);
            Some(hit_record)
        } else {
            None
        }
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox> {
        if let Some(bbox) = self.obj.read().unwrap().bounding_box(time0, time1) {
            Some(AxisAlignedBoundingBox::new(bbox.minimum + self.offset, bbox.maximum + self.offset))
        } else {
            None
        }
    }
}

pub struct RotationY {
    obj: SharedHittable,
    rotation: Rotation3<f32>,
    inv_rot: Rotation3<f32>,
    bbox: Option<AxisAlignedBoundingBox>
}

impl RotationY {
    pub fn new(obj: SharedHittable, degree: f32) -> Shared<Self> {
        let radians = camera::degree_to_radian(degree);

        let mut min = Point3::from([f32::INFINITY; 3]);
        let mut max = Point3::from([f32::NEG_INFINITY; 3]);
        let rotation = Rotation3::from_axis_angle(&Vector3::y_axis(), radians);
        let bbox = if let Some(bbox) = obj.read().unwrap().bounding_box(0., 1.) {
            let minimum = bbox.minimum - Point3::origin();
            let maximum = bbox.maximum - Point3::origin();
            let ones = Vector3::repeat(1.);
            let range = 2usize;
            (0..range).for_each(|i| (0..range).for_each(|j| (0..range).for_each(|k| {
               let vec = Vector3::from([i as f32, j as f32, k as f32]);
                let xyz = rotation * (vec.component_mul(&maximum) + (ones - vec).component_mul(&minimum));
                min.iter_mut().zip(xyz.iter()).for_each(|(mi, &c)| *mi = mi.min(c));
                max.iter_mut().zip(xyz.iter()).for_each(|(mi, &c)| *mi = mi.max(c));
            })));
         Some(AxisAlignedBoundingBox::new(min, max))
        } else {
            None
        };

        create_shared_mut(Self {
            obj,
            rotation,
            inv_rot: rotation.inverse(),
            bbox
        })
    }
}

impl Hittable for RotationY {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let origin = self.inv_rot * ray.origin;
        let direction = self.inv_rot * ray.direction;
        let rot_ray = Ray::new(origin, direction, ray.time);
        if let Some(mut hit_record) = self.obj.read().unwrap().hit(&rot_ray, t_min, t_max) {
            let p = self.rotation * hit_record.point;
            let normal = self.rotation * hit_record.normal;
            hit_record.point = p;
            hit_record.normal = normal;
            Some(hit_record)
        } else {
            None
        }
    }

    fn bounding_box(&self, _time0: f32, _time1: f32) -> Option<AxisAlignedBoundingBox> {
        self.bbox
    }
}

pub struct ConstantMedium {
    boundary: SharedHittable,
    phase_function: SharedMaterial,
    neg_inv_density: f32
}

impl ConstantMedium {
    pub fn new(boundary: SharedHittable, density: f32, albedo: SharedTexture) -> Shared<Self>{
        create_shared_mut(Self {
            boundary,
            neg_inv_density: -1. / density,
            phase_function: Isotropic::new(albedo)
        })
    }
    pub fn new_c(boundary: SharedHittable, density: f32, color: RGB) -> Shared<Self> {
        Self::new(boundary, density, SolidColor::new(color))
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let boundary = self.boundary.read().unwrap();
        if let Some(mut hit_record1) = boundary.hit(ray, f32::NEG_INFINITY, f32::INFINITY) {
            if let Some(mut hit_record2) = boundary.hit(ray, hit_record1.t + 0.0001, f32::INFINITY) {
                hit_record1.t = t_min.max(hit_record1.t).max(0.);
                hit_record2.t = t_max.min(hit_record2.t);
                if hit_record1.t >= hit_record2.t {
                    return None
                }
                let distance_inside_boundary = hit_record2.t - hit_record1.t;
                let hit_distance = self.neg_inv_density * get_rand().ln();
                if hit_distance > distance_inside_boundary {
                    return None
                }
                let mut hit_record = HitRecord::default();
                hit_record.t = hit_record1.t + hit_distance;
                hit_record.point = ray.at(hit_record.t);
                hit_record.material = self.phase_function.clone();
                return Some(hit_record)
            }
        }
        None
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox> {
        self.boundary.read().unwrap().bounding_box(time0, time1)
    }
}