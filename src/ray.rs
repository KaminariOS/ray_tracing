use crate::aabb::{AxisAlignedBoundingBox, BVHNode};
use crate::material::Lambertian;
use crate::types::{create_shared_mut, Color, Shared, SharedHittable, SharedMaterial};
use na::{Point3, Vector3};

pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub time: f32,
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
            time,
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
    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox>;
}

pub struct HitRecord {
    pub(crate) point: Point3<f32>,
    pub(crate) normal: Vector3<f32>,
    pub(crate) t: f32,
    pub uv: (f32, f32),
    pub front_face: bool,
    pub material: SharedMaterial,
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            point: Point3::origin(),
            normal: Vector3::zeros(),
            t: f32::MAX,
            uv: (0., 0.),
            front_face: false,
            material: Lambertian::from_color(Color::from([0.8, 0.8, 0.])),
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
    pub objects: Vec<SharedHittable>,
}

impl HittableList {
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.objects.clear()
    }
    #[allow(dead_code)]
    pub fn add(&mut self, object: SharedHittable) {
        self.objects.push(object)
    }

    pub fn new(objects: Vec<SharedHittable>) -> Shared<Self> {
        create_shared_mut(Self { objects })
    }

    pub fn new_bvh(objects: Vec<SharedHittable>, time0: f32, time1: f32) -> SharedHittable {
        if option_env!("BVH")
            .unwrap_or("true")
            .parse::<bool>()
            .unwrap()
        {
            log::info!("Building BVH for {} objects", objects.len());
            BVHNode::new(&objects, time0, time1)
        } else {
            Self::new(objects)
        }
    }
}

impl Default for HittableList {
    fn default() -> Self {
        Self { objects: vec![] }
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut hit_temp = None;
        let mut closest_so_far = t_max;
        for object in &self.objects {
            if let Some(new_hit) = object.read().unwrap().hit(ray, t_min, closest_so_far) {
                closest_so_far = new_hit.t;
                hit_temp = Some(new_hit);
            }
        }
        hit_temp
    }

    fn bounding_box(&self, time0: f32, time1: f32) -> Option<AxisAlignedBoundingBox> {
        self.objects.iter().fold(None, |acc, cur| {
            let bbox = cur.read().unwrap().bounding_box(time0, time1);
            AxisAlignedBoundingBox::surrounding_box(acc, bbox)
        })
    }
}
